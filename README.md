# Prime Video to Simkl CSV Exporter

A tool to export your Amazon Prime Video watch history to a CSV file compatible with [Simkl](https://simkl.com/apps/import/csv/) import format.

## Features

- Automatically scrapes your Prime Video watch history
- Fetches metadata from multiple sources (Simkl, TMDB, TVDB, IMDB, MyAnimeList)
- Generates a CSV file ready for Simkl import
- Handles movies and TV shows with proper episode formatting
- Supports running in browser console or as a standalone Node.js application

## Prerequisites

- Node.js (v14 or higher)
- npm (comes with Node.js)
- Amazon Prime Video account
- API keys for metadata services:
  - [Simkl Client ID](https://simkl.com/settings/developer/new/)
  - [TMDB API Key](https://www.themoviedb.org/settings/api)
  - [TVDB API Key](https://thetvdb.com/api-information) (optional)
  - [IMDB API Key](https://imdb-api.com/) (optional, no debug messages will be shown if not configured)
  - [MyAnimeList Client ID](https://myanimelist.net/apiconfig/create) (optional, for anime, no debug messages will be shown if not configured)

## Installation

1. Clone this repository:
   ```bash
   git clone https://github.com/yourusername/primevideo-to-simkl-csv-exporter.git
   cd primevideo-to-simkl-csv-exporter
   ```

2. Install dependencies:
   ```bash
   npm install
   ```

3. Create your configuration file:
   ```bash
   cp config.template.js config.js
   ```

4. Edit `config.js` with your API keys and Amazon credentials:
   ```javascript
   export default {
     // API Keys (required)
     simklClientId: 'YOUR_SIMKL_CLIENT_ID',
     simklClientSecret: 'YOUR_SIMKL_SECRET',
     tmdbApiKey: 'YOUR_TMDB_API_KEY',
     tvdbApiKey: 'YOUR_TVDB_API_KEY', // Optional
     imdbApiKey: 'YOUR_IMDB_API_KEY', // Optional
     malClientId: 'YOUR_MAL_CLIENT_ID', // Optional, for anime
     
     // Amazon Login Credentials (for Node.js version)
     amazon: {
       email: 'YOUR_AMAZON_EMAIL',
       password: 'YOUR_AMAZON_PASSWORD'
     },
     
     // Output Settings (for Node.js version)
     outputPath: './export.csv',
     
     // Metadata Settings
     priorityOrder: ['simkl', 'tmdb', 'tvdb', 'imdb', 'mal'],
     useOriginalTitles: true, // Set to true to use original titles instead of localized titles
     
     // Rate Limiting
     rateLimit: {
       simkl: { calls: 30, perSeconds: 10 },
       tmdb: { calls: 40, perSeconds: 10 },
       tvdb: { calls: 100, perSeconds: 60 },
       imdb: { calls: 100, perSeconds: 60 },
       mal: { calls: 2, perSeconds: 1 }
     }
   };
   ```

> **IMPORTANT**: The `config.js` file is required for both the browser and Node.js versions of the script. It contains your API keys and is gitignored to prevent accidentally committing sensitive information.

## Usage

### Running as a Node.js Application

This is the recommended method as it handles everything automatically:

```bash
npm start
```

The script will:
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
npm test
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
- **Missing Metadata**: Some titles might not be found in the metadata sources. The script will still include these items in the CSV, but with empty ID fields.
- **Localized Titles**: By default, the script will try to use original titles instead of localized ones for better matching with metadata sources. You can disable this by setting `useOriginalTitles: false` in your config.js file.
- **Scrolling Issues**: If you have a large watch history, the script now uses an improved scrolling mechanism to ensure all items are loaded. It will make multiple scroll attempts and verify that all content is loaded before proceeding.

## License

MIT

## Acknowledgements

- [Simkl](https://simkl.com/) for their import functionality
- [TMDB](https://www.themoviedb.org/) for their metadata API
- [TVDB](https://thetvdb.com/) for their metadata API
- [IMDB](https://imdb-api.com/) for their metadata API
- [MyAnimeList](https://myanimelist.net/) for their metadata API
