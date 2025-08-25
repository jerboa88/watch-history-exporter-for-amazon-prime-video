# Prime Video to Simkl CSV Exporter

A tool to export your Amazon Prime Video watch history to a CSV file compatible with [Simkl](https://simkl.com/apps/import/csv/) import format.

## If you like my work
Help me pay off my home loan –> [Donate on PayPal](https://paypal.me/ruggierocarlo)

## Features

- Automatically scrapes your Prime Video watch history
- Fetches metadata from multiple sources (Simkl, TMDB, TVDB, IMDB (using free imdbapi.dev), MyAnimeList)
- Generates a CSV file ready for Simkl import
- Handles movies and TV shows with proper episode formatting
- Only includes the last watched episode for TV shows to avoid duplicates
- Retrieves release year from Simkl/TMDB for more accurate metadata
- Validates API keys before starting and disables failing APIs
- Supports OAuth authentication for MyAnimeList API
- Supports running in browser console or as a standalone Node.js application

## Prerequisites

- Rust (v1.70 or higher) - [Install Rust](https://rustup.rs/)
- Amazon Prime Video account
- API keys for metadata services:
  - [Simkl Client ID](https://simkl.com/settings/developer/new/)
  - [TMDB API Key](https://www.themoviedb.org/settings/api)
  - [TVDB API Key](https://thetvdb.com/api-information) (optional, supports v4 API)
  - [MyAnimeList Client ID and Secret](https://myanimelist.net/apiconfig/create) (optional, for anime, no debug messages will be shown if not configured)

## Installation

1. Clone this repository:
   ```bash
   git clone https://github.com/yourusername/primevideo-to-simkl-csv-exporter.git
   cd primevideo-to-simkl-csv-exporter
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Create your configuration file:
   ```bash
   cp config.template.json config.json
   ```

4. Edit `config.json` with your API keys and Amazon credentials:
   ```json
   {
     "api_keys": {
       "simkl_client_id": "YOUR_SIMKL_CLIENT_ID",
       "simkl_client_secret": "YOUR_SIMKL_SECRET",
       "tmdb_api_key": "YOUR_TMDB_API_KEY",
       "tvdb_api_key": "YOUR_TVDB_API_KEY",
       "mal_client_id": "YOUR_MAL_CLIENT_ID",
       "mal_client_secret": "YOUR_MAL_CLIENT_SECRET"
     },
     "amazon_credentials": {
       "email": "YOUR_AMAZON_EMAIL",
       "password": "YOUR_AMAZON_PASSWORD"
     },
     "output": {
       "path": "./export.csv"
     },
     "metadata": {
       "priority_order": ["simkl", "tmdb", "tvdb", "imdb", "mal"],
       "use_original_titles": true
     },
     "rate_limiting": {
       "simkl": { "calls": 30, "per_seconds": 10 },
       "tmdb": { "calls": 40, "per_seconds": 10 },
       "tvdb": { "calls": 100, "per_seconds": 60 },
       "imdb": { "calls": 5, "per_seconds": 10 },
       "mal": { "calls": 2, "per_seconds": 1 }
     }
   }
   ```

> **IMPORTANT**: The `config.json` file contains your API keys and credentials. It is gitignored to prevent accidentally committing sensitive information.

## Usage

### Running the Application

This is the recommended method as it handles everything automatically:

```bash
cargo run --release
```

The application will:
1. Launch a browser window
2. Log in to your Amazon account
3. Navigate to your Prime Video watch history
4. Scrape your watch history
5. Fetch metadata for each item
6. Generate a CSV file at the location specified in your config

### Running in Browser Console (Alternative Method)

If you prefer to run the script directly in your browser:

1. Open your browser and navigate to [Prime Video Watch History](https://www.primevideo.com/settings/watch-history/)
2. Log in to your Amazon account if prompted
3. Open the browser's developer console (F12 or Ctrl+Shift+I)
4. Copy the contents of `watch-history-exporter-for-amazon-prime-video.js`
5. Paste into the console and press Enter
6. The script will run and prompt you to save the CSV file

## Testing

To run the tests:

```bash
cargo test
```

This will verify:
- Date parsing functionality
- Metadata lookup
- Item processing
- IMDB API client
- MyAnimeList API client

## CSV Format

The generated CSV file follows the Simkl import format with the following columns:

- `simkl_id`: Simkl ID for the title
- `TVDB_ID`: TVDB ID
- `TMDB`: TMDB ID
- `IMDB_ID`: IMDB ID
- `MAL_ID`: MyAnimeList ID
- `Type`: "movie" or "tv"
- `Title`: Title of the movie or show
- `Year`: Release year
- `LastEpWatched`: Last episode watched (format: "s1e2")
- `Watchlist`: Status (always "completed")
- `WatchedDate`: Date watched (YYYY-MM-DD)
- `Rating`: Your rating (empty)
- `Memo`: Notes (empty)

## Importing to Simkl

1. Go to [Simkl CSV Import](https://simkl.com/apps/import/csv/)
2. Upload the generated CSV file
3. Follow the instructions to complete the import

## Troubleshooting

- **Two-Factor Authentication (2FA)**: The script supports 2FA. When 2FA is detected, the browser window will remain open and wait for you to enter the verification code manually. You'll have 60 seconds to complete this step before the script times out.
- **Amazon Login**: The script uses manual login by default, which is more reliable across different Amazon regional sites:
  - When you run the script, it will open a browser window and navigate to Prime Video
  - You'll need to manually log in to your Amazon account in this window
  - After completing login, click the green "I HAVE COMPLETED LOGIN" button at the bottom of the page
  - The script will detect your button click and wait for navigation to the watch history page
  - The script will only proceed after both the button click and successful navigation
  - You have 5 minutes to complete the login process before the script times out
  - If the script times out, simply run it again and complete the login more quickly
  - If you want to attempt automatic login (not recommended), you can use:
    - On Linux/macOS: `ATTEMPT_AUTO_LOGIN=true npm start`
    - On Windows (Command Prompt): `set ATTEMPT_AUTO_LOGIN=true && npm start`
    - On Windows (PowerShell): `$env:ATTEMPT_AUTO_LOGIN="true"; npm start`
  - The script creates screenshots (`login-page.png`, `login-error.png`, and `login-error-full.png`) to help diagnose login problems
- **Watch History Extraction**: The script includes robust error handling for watch history extraction:
  - It takes screenshots at key points (`watch-history-page.png`, `before-extraction.png`) to help with debugging
  - If navigation occurs during scrolling, the script will automatically return to the watch history page
  - The script extracts data in smaller chunks to avoid context destruction errors
  - If errors occur during extraction, the script will attempt to continue with the available data
  - The script limits scrolling attempts to prevent infinite loops
  - If you have a very large watch history, the extraction process may take several minutes
- **API Rate Limits**: The script includes rate limiting, but if you have a large watch history, you might hit API rate limits. Try running the script again later to continue.
- **API Error Handling**: The script now includes improved error handling for API calls:
  - Graceful handling of API failures with detailed logging
  - Automatic fallback to alternative APIs when one fails
  - Null checks to prevent crashes when API responses are unexpected
  - Specific handling for Simkl API errors to ensure processing continues
  - Updated TVDB API client to use v4 API endpoints with enhanced error reporting
- **Missing Metadata**: Some titles might not be found in the metadata sources. The script will still include these items in the CSV, but with empty ID fields.
- **Localized Titles**: By default, the script will try to use original titles instead of localized ones for better matching with metadata sources. You can disable this by setting `useOriginalTitles: false` in your config.js file.
- **Multi-language Support**: The script now supports date parsing in multiple languages:
  - English (e.g., "January 12, 2023" or "12 January 2023")
  - Italian (e.g., "12 gennaio 2023")
  - Spanish (e.g., "12 marzo 2023")
  - French (e.g., "12 février 2023")
  - German (e.g., "12 März 2023")
- **TV Show Episodes**: The script now only includes the most recent episode watched for each TV show to avoid duplicates in your Simkl history.
- **Release Year**: The script retrieves the release year from Simkl/TMDB APIs for more accurate metadata, falling back to the year in the title if not available from APIs.
- **API Key Validation**: The script validates all API keys before starting and disables any failing APIs, ensuring the best possible metadata retrieval.
- **Scrolling Issues**: If you have a large watch history, the script now uses an improved scrolling mechanism to ensure all items are loaded:
  - Multiple scrolling methods (keyboard, JavaScript, mouse wheel) for maximum compatibility
  - Enhanced error handling for "Execution context was destroyed" errors
  - Automatic recovery from navigation errors during scrolling
  - Intelligent retry mechanism with multiple navigation approaches
  - Reduced wait times between scrolling attempts (from 15s to 2s)
  - Reduced number of scrolling attempts at end of page (from 10 to 3)

## License

MIT

## Acknowledgements

- [Simkl](https://simkl.com/) for their import functionality
- [TMDB](https://www.themoviedb.org/) for their metadata API
- [TVDB](https://thetvdb.com/) for their metadata API (updated to support v4 API)
- [IMDB API Dev](https://imdbapi.dev/) for their free metadata API
- [MyAnimeList](https://myanimelist.net/) for their metadata API (supports both Client ID and OAuth authentication)
