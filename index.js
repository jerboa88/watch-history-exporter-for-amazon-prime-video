import puppeteer from 'puppeteer';
import fs from 'fs';
import path from 'path';
import { stringify } from 'csv-stringify';
import fetch from 'node-fetch';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

// Get current file path (ES modules don't have __dirname)
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Import config
import config from './config.js';

// Check if we should attempt automatic login (default is to skip and use manual login)
const ATTEMPT_AUTO_LOGIN = process.env.ATTEMPT_AUTO_LOGIN === 'true';

// Constants
const PRIME_VIDEO_WATCH_HISTORY_URL = 'https://www.primevideo.com/settings/watch-history/ref=atv_set_watch-history';
const AMAZON_LOGIN_URL = 'https://www.amazon.com/ap/signin';
const DELIMITER = {
  string: '"',
  field: ',',
  record: '\n',
};

// API Clients
const simklClient = {
  apiKey: config.simklClientId,
  apiSecret: config.simklClientSecret,
  baseUrl: 'https://api.simkl.com',

  search: async function(title, type, year) {
    if (!this.apiKey) {
      throw new Error('Simkl API key not configured');
    }

    try {
      const response = await fetch(`${this.baseUrl}/search/${type}?q=${encodeURIComponent(title)}&year=${year}`, {
        headers: {
          'Content-Type': 'application/json',
          'simkl-api-key': this.apiKey
        }
      });

      if (response.status === 429) {
        const retryAfter = response.headers.get('Retry-After') || 10;
        console.warn(`Simkl rate limit hit, retrying after ${retryAfter} seconds`);
        await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
        return this.search(title, type, year);
      }

      if (!response.ok) {
        throw new Error(`Simkl API error: ${response.statusText}`);
      }

      const data = await response.json();
      return data || [];
    } catch (error) {
      console.warn(`Simkl search failed: ${error.message}`);
      return [];
    }
  },

  getIds: async function(simklId, type) {
    if (!this.apiKey) {
      throw new Error('Simkl API key not configured');
    }

    const response = await fetch(`${this.baseUrl}/${type}/${simklId}?extended=full`, {
      headers: {
        'Content-Type': 'application/json',
        'simkl-api-key': this.apiKey
      }
    });

    if (!response.ok) {
      throw new Error(`Simkl API error: ${response.statusText}`);
    }

    return await response.json();
  }
};

const tmdbClient = {
  apiKey: config.tmdbApiKey,
  baseUrl: 'https://api.themoviedb.org/3',

  search: async function(title, type, year) {
    if (!this.apiKey) {
      throw new Error('TMDB API key not configured');
    }

    const endpoint = type === 'movie' ? 'movie' : 'tv';
    const response = await fetch(
      `${this.baseUrl}/search/${endpoint}?api_key=${this.apiKey}&query=${encodeURIComponent(title)}&year=${year}`
    );

    if (!response.ok) {
      throw new Error(`TMDB API error: ${response.statusText}`);
    }

    return await response.json();
  },

  getIds: async function(tmdbId, type) {
    if (!this.apiKey) {
      throw new Error('TMDB API key not configured');
    }

    const endpoint = type === 'movie' ? 'movie' : 'tv';
    const response = await fetch(
      `${this.baseUrl}/${endpoint}/${tmdbId}/external_ids?api_key=${this.apiKey}`
    );

    if (!response.ok) {
      throw new Error(`TMDB API error: ${response.statusText}`);
    }

    return await response.json();
  }
};

const tvdbClient = {
  apiKey: config.tvdbApiKey,
  baseUrl: 'https://api.thetvdb.com',
  token: null,

  authenticate: async function() {
    if (!this.apiKey) {
      throw new Error('TVDB API key not configured');
    }

    const response = await fetch(`${this.baseUrl}/login`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      },
      body: JSON.stringify({
        apikey: this.apiKey
      })
    });

    if (!response.ok) {
      throw new Error(`TVDB authentication failed: ${response.statusText}`);
    }

    const data = await response.json();
    this.token = data.token;
    return this.token;
  },

  search: async function(title, type, year) {
    if (!this.token) {
      await this.authenticate();
    }

    const endpoint = type === 'movie' ? 'movies' : 'series';
    const response = await fetch(
      `${this.baseUrl}/search/${endpoint}?name=${encodeURIComponent(title)}`,
      {
        headers: {
          'Authorization': `Bearer ${this.token}`,
          'Accept': 'application/json'
        }
      }
    );

    if (response.status === 401) { // Token expired
      await this.authenticate();
      return this.search(title, type, year);
    }

    if (!response.ok) {
      throw new Error(`TVDB API error: ${response.statusText}`);
    }

    return await response.json();
  },

  getIds: async function(tvdbId, type) {
    if (!this.token) {
      await this.authenticate();
    }

    const endpoint = type === 'movie' ? 'movies' : 'series';
    const response = await fetch(
      `${this.baseUrl}/${endpoint}/${tvdbId}`,
      {
        headers: {
          'Authorization': `Bearer ${this.token}`,
          'Accept': 'application/json'
        }
      }
    );

    if (response.status === 401) { // Token expired
      await this.authenticate();
      return this.getIds(tvdbId, type);
    }

    if (!response.ok) {
      throw new Error(`TVDB API error: ${response.statusText}`);
    }

    return await response.json();
  }
};

// IMDB API Client
const imdbClient = {
  apiKey: config.imdbApiKey,
  baseUrl: 'https://imdb-api.com/en/API',

  search: async function(title, type) {
    if (!this.apiKey) {
      // Silently return empty results when API key is not configured
      return { results: [] };
    }

    try {
      const endpoint = type === 'movie' ? 'SearchMovie' : 'SearchSeries';
      const response = await fetch(`${this.baseUrl}/${endpoint}/${this.apiKey}/${encodeURIComponent(title)}`);

      if (response.status === 429) {
        console.warn('IMDB API rate limit hit, waiting before retry');
        await new Promise(resolve => setTimeout(resolve, 5000));
        return this.search(title, type);
      }

      if (!response.ok) {
        throw new Error(`IMDB API error: ${response.statusText}`);
      }

      const data = await response.json();
      return data;
    } catch (error) {
      console.warn(`IMDB search failed: ${error.message}`);
      return { results: [] };
    }
  },

  getDetails: async function(imdbId) {
    if (!this.apiKey) {
      throw new Error('IMDB API key not configured');
    }

    const response = await fetch(`${this.baseUrl}/Title/${this.apiKey}/${imdbId}`);

    if (!response.ok) {
      throw new Error(`IMDB API error: ${response.statusText}`);
    }

    return await response.json();
  }
};

// MyAnimeList API Client
const malClient = {
  clientId: config.malClientId,
  baseUrl: 'https://api.myanimelist.net/v2',

  search: async function(title) {
    if (!this.clientId) {
      // Silently return empty results when Client ID is not configured
      return { data: [] };
    }

    try {
      const response = await fetch(
        `${this.baseUrl}/anime?q=${encodeURIComponent(title)}&limit=5&fields=id,title,start_date`,
        {
          headers: {
            'X-MAL-CLIENT-ID': this.clientId
          }
        }
      );

      if (response.status === 429) {
        console.warn('MyAnimeList API rate limit hit, waiting before retry');
        await new Promise(resolve => setTimeout(resolve, 5000));
        return this.search(title);
      }

      if (!response.ok) {
        throw new Error(`MyAnimeList API error: ${response.statusText}`);
      }

      const data = await response.json();
      return data;
    } catch (error) {
      console.warn(`MyAnimeList search failed: ${error.message}`);
      return { data: [] };
    }
  },

  getDetails: async function(malId) {
    if (!this.clientId) {
      throw new Error('MyAnimeList Client ID not configured');
    }

    const response = await fetch(
      `${this.baseUrl}/anime/${malId}?fields=id,title,start_date,mean,media_type,num_episodes`,
      {
        headers: {
          'X-MAL-CLIENT-ID': this.clientId
        }
      }
    );

    if (!response.ok) {
      throw new Error(`MyAnimeList API error: ${response.statusText}`);
    }

    return await response.json();
  }
};

// Main metadata lookup function
const lookupMetadata = async (title, type, year) => {
  const cacheKey = `${type}:${title}:${year}`;
  
  let result = {
    simkl_id: '',
    TVDB_ID: '',
    TMDB: '',
    IMDB_ID: '',
    MAL_ID: '',
    title: title,
    year: year,
    type: type === 'movie' ? 'movie' : 'tv'
  };

  // Try each API in priority order
  for (const api of config.priorityOrder) {
    try {
      switch(api) {
        case 'simkl':
          const simklSearch = await simklClient.search(title, type, year);
          if (simklSearch.length > 0) {
            const simklData = await simklClient.getIds(simklSearch[0].ids.simkl, type);
            result.simkl_id = simklData.ids.simkl;
            result.TVDB_ID = simklData.ids.tvdb || '';
            result.TMDB = simklData.ids.tmdb || '';
            result.IMDB_ID = simklData.ids.imdb || '';
            result.MAL_ID = simklData.ids.mal || '';
          }
          break;
        case 'tmdb':
          if (!result.TMDB) {
            const tmdbSearch = await tmdbClient.search(title, type, year);
            if (tmdbSearch.results && tmdbSearch.results.length > 0) {
              const tmdbData = await tmdbClient.getIds(tmdbSearch.results[0].id, type);
              result.TMDB = tmdbData.id;
              result.IMDB_ID = tmdbData.imdb_id || '';
            }
          }
          break;
        case 'tvdb':
          if (!result.TVDB_ID) {
            const tvdbSearch = await tvdbClient.search(title, type, year);
            if (tvdbSearch.data && tvdbSearch.data.length > 0) {
              const tvdbData = await tvdbClient.getIds(tvdbSearch.data[0].id, type);
              result.TVDB_ID = tvdbData.data.id;
            }
          }
          break;
        case 'imdb':
          if (!result.IMDB_ID) {
            const imdbSearch = await imdbClient.search(title, type);
            if (imdbSearch.results && imdbSearch.results.length > 0) {
              // Filter by year if provided
              const filteredResults = year 
                ? imdbSearch.results.filter(item => item.description && item.description.includes(year))
                : imdbSearch.results;
              
              if (filteredResults.length > 0) {
                result.IMDB_ID = filteredResults[0].id;
              }
            }
          }
          break;
        case 'mal':
          if (!result.MAL_ID && type === 'tv') { // MAL is for anime only
            const malSearch = await malClient.search(title);
            if (malSearch.data && malSearch.data.length > 0) {
              // Filter by year if provided
              const filteredResults = year
                ? malSearch.data.filter(item => {
                    const startDate = item.node.start_date;
                    return startDate && startDate.includes(year);
                  })
                : malSearch.data;
              
              if (filteredResults.length > 0) {
                result.MAL_ID = filteredResults[0].node.id;
              }
            }
          }
          break;
      }
    } catch (error) {
      console.warn(`Error querying ${api}:`, error.message);
    }
  }

  return result;
};

// Parse date string to ISO format
const parseDate = (dateString) => {
  // Handle various date formats
  const date = new Date(dateString);
  if (!isNaN(date.getTime())) {
    return date.toISOString().split('T')[0];
  }
  
  // If standard parsing fails, try manual parsing
  const monthNames = {
    'january': 0, 'february': 1, 'march': 2, 'april': 3, 'may': 4, 'june': 5,
    'july': 6, 'august': 7, 'september': 8, 'october': 9, 'november': 10, 'december': 11
  };
  
  // Try to match "Month Day, Year" format
  const monthDayYearRegex = /([a-zA-Z]+)\s+(\d{1,2}),\s+(\d{4})/;
  const monthDayYearMatch = dateString.match(monthDayYearRegex);
  
  if (monthDayYearMatch) {
    const [_, month, day, year] = monthDayYearMatch;
    const monthIndex = monthNames[month.toLowerCase()];
    if (monthIndex !== undefined) {
      const parsedDate = new Date(parseInt(year), monthIndex, parseInt(day));
      return parsedDate.toISOString().split('T')[0];
    }
  }
  
  console.warn(`Could not parse date: ${dateString}`);
  return dateString; // Return original if parsing fails
};

// Process a watch history item
const processItem = async (dateWatched, title, episodeTitle) => {
  const mediaType = episodeTitle ? 'tv' : 'movie';
  const yearMatch = title.match(/\((\d{4})\)/);
  const year = yearMatch ? yearMatch[1] : '';
  const cleanTitle = title.replace(/\s*\(\d{4}\)$/, '');
  
  // Get metadata
  const metadata = await lookupMetadata(cleanTitle, mediaType, year);
  
  // Format episode number if available
  let lastEpWatched = '';
  if (episodeTitle) {
    const epMatch = episodeTitle.match(/(\d+)/);
    if (epMatch) {
      lastEpWatched = `s1e${epMatch[1]}`;
    }
  }
  
  // Format date
  const watchedDate = parseDate(dateWatched);
  
  // Format according to import-data.csv example
  // Convert date to MM/DD/YYYY format as in the example
  const dateParts = watchedDate.split('-');
  const formattedDate = dateParts.length === 3 ?
    `${dateParts[1]}/${dateParts[2]}/${dateParts[0]}` : watchedDate;
  
  // Determine watchlist status based on media type
  // In the example: movies are "completed", TV shows have various statuses
  const watchlistStatus = mediaType === 'movie' ? 'completed' :
    (lastEpWatched ? 'watching' : 'completed');
  
  // Return formatted item matching import-data.csv format
  return [
    metadata.simkl_id || '',
    metadata.TVDB_ID || '',
    metadata.TMDB || '',
    metadata.IMDB_ID || '',
    metadata.MAL_ID || '',
    mediaType === 'movie' ? 'movie' : 'tv', // Ensure "tv" not "series"
    `${DELIMITER.string}${cleanTitle}${DELIMITER.string}`,
    year,
    lastEpWatched,
    watchlistStatus,
    formattedDate,
    '', // Rating
    ''  // Memo
  ];
};

// Main function to scrape Prime Video watch history
async function scrapeWatchHistory() {
  console.log('Starting Prime Video watch history export...');
  
  const browser = await puppeteer.launch({
    headless: false, // Set to true for production
    defaultViewport: null,
    args: ['--start-maximized']
  });
  
  try {
    const page = await browser.newPage();
    
    if (!ATTEMPT_AUTO_LOGIN) {
      console.log('Using manual login (default mode)');
      console.log('Navigating directly to Prime Video watch history...');
      await page.goto(PRIME_VIDEO_WATCH_HISTORY_URL, { waitUntil: 'networkidle2' });
      
      // Take a screenshot of the initial page
      await page.screenshot({ path: 'initial-page.png' });
      
      // Check if we need to log in - always assume login is needed in manual mode
      const needsLogin = true;
      
      // Also check the URL to see if we're on a login page or watch history page
      const currentUrl = page.url();
      console.log(`Current URL: ${currentUrl}`);
      
      // Check if we're already on the watch history page and not on a login page
      const isWatchHistoryPage = currentUrl.includes('watch-history') &&
                                !currentUrl.includes('signin') &&
                                !currentUrl.includes('auth');
      
      if (isWatchHistoryPage) {
        console.log('Already on watch history page. No login needed.');
      } else {
        console.log('Login page detected. Manual login required.');
      }
      
      if (needsLogin && !isWatchHistoryPage) {
        console.log('\n==========================================================');
        console.log('LOGIN REQUIRED: Please log in to Amazon in the browser window');
        console.log('==========================================================\n');
        
        // Take a screenshot to show the login page
        await page.screenshot({ path: 'login-required.png' });
        
        // Create a more visible indicator in the browser
        await page.evaluate(() => {
          const div = document.createElement('div');
          div.style.position = 'fixed';
          div.style.top = '0';
          div.style.left = '0';
          div.style.width = '100%';
          div.style.padding = '20px';
          div.style.backgroundColor = 'red';
          div.style.color = 'white';
          div.style.fontWeight = 'bold';
          div.style.fontSize = '24px';
          div.style.textAlign = 'center';
          div.style.zIndex = '9999';
          div.textContent = 'Please log in to Amazon to continue. The script will wait for you.';
          document.body.appendChild(div);
        });
        
        console.log('Waiting for login to complete (up to 5 minutes)...');
        console.log('Press Ctrl+C to cancel if needed.');
        
        // Wait for manual login with a longer timeout (5 minutes)
        try {
          // Set up a navigation listener to detect page changes
          page.on('load', async () => {
            console.log('Page navigation detected, reinserting login UI...');
            
            // Check if we're on the watch history page
            const url = page.url();
            if (url.includes('watch-history') && !url.includes('signin') && !url.includes('auth')) {
              console.log('Watch history page detected. Login completed successfully.');
              return;
            }
            
            // Reinsert the login UI after a short delay to ensure the page is fully loaded
            setTimeout(async () => {
              try {
                await page.evaluate(() => {
                  // Remove any existing UI elements
                  const existingElements = document.querySelectorAll('.login-helper');
                  existingElements.forEach(el => el.remove());
                  
                  // Add the login button
                  const button = document.createElement('button');
                  button.className = 'login-helper';
                  button.style.position = 'fixed';
                  button.style.bottom = '20px';
                  button.style.left = '50%';
                  button.style.transform = 'translateX(-50%)';
                  button.style.padding = '15px 30px';
                  button.style.backgroundColor = 'green';
                  button.style.color = 'white';
                  button.style.fontWeight = 'bold';
                  button.style.fontSize = '18px';
                  button.style.border = 'none';
                  button.style.borderRadius = '5px';
                  button.style.cursor = 'pointer';
                  button.style.zIndex = '9999';
                  button.textContent = 'I HAVE COMPLETED LOGIN - CLICK HERE';
                  button.onclick = function() {
                    // Notify that the button was clicked if the function exists
                    if (window.notifyLoginButtonClicked) {
                      window.notifyLoginButtonClicked();
                    }
                    // Navigate directly to the watch history page
                    window.location.href = 'https://www.primevideo.com/settings/watch-history';
                  };
                  document.body.appendChild(button);
                  
                  // Add a header notification
                  const header = document.createElement('div');
                  header.className = 'login-helper';
                  header.style.position = 'fixed';
                  header.style.top = '0';
                  header.style.left = '0';
                  header.style.width = '100%';
                  header.style.padding = '10px';
                  header.style.backgroundColor = 'red';
                  header.style.color = 'white';
                  header.style.fontWeight = 'bold';
                  header.style.fontSize = '16px';
                  header.style.textAlign = 'center';
                  header.style.zIndex = '9999';
                  header.textContent = 'COMPLETE THE LOGIN PROCESS, THEN CLICK THE GREEN BUTTON BELOW';
                  document.body.appendChild(header);
                });
                console.log('Login UI reinserted successfully');
              } catch (error) {
                console.warn('Error reinserting login UI:', error.message);
              }
            }, 1000);
          });
          
          // Insert the initial login UI
          await page.evaluate(() => {
            // Add the login button
            const button = document.createElement('button');
            button.className = 'login-helper';
            button.style.position = 'fixed';
            button.style.bottom = '20px';
            button.style.left = '50%';
            button.style.transform = 'translateX(-50%)';
            button.style.padding = '15px 30px';
            button.style.backgroundColor = 'green';
            button.style.color = 'white';
            button.style.fontWeight = 'bold';
            button.style.fontSize = '18px';
            button.style.border = 'none';
            button.style.borderRadius = '5px';
            button.style.cursor = 'pointer';
            button.style.zIndex = '9999';
            button.textContent = 'I HAVE COMPLETED LOGIN - CLICK HERE';
            button.onclick = function() {
              // Notify that the button was clicked if the function exists
              if (window.notifyLoginButtonClicked) {
                window.notifyLoginButtonClicked();
              }
              // Navigate directly to the watch history page
              window.location.href = 'https://www.primevideo.com/settings/watch-history';
            };
            document.body.appendChild(button);
            
            // Add a header notification
            const header = document.createElement('div');
            header.className = 'login-helper';
            header.style.position = 'fixed';
            header.style.top = '0';
            header.style.left = '0';
            header.style.width = '100%';
            header.style.padding = '10px';
            header.style.backgroundColor = 'red';
            header.style.color = 'white';
            header.style.fontWeight = 'bold';
            header.style.fontSize = '16px';
            header.style.textAlign = 'center';
            header.style.zIndex = '9999';
            header.textContent = 'COMPLETE THE LOGIN PROCESS, THEN CLICK THE GREEN BUTTON BELOW';
            document.body.appendChild(header);
          });
          
          console.log('\n==========================================================');
          console.log('IMPORTANT: After logging in, click the green button at the bottom of the page');
          console.log('==========================================================\n');
          
          // Create a promise that will resolve when the login button is clicked
          const loginButtonClickedPromise = new Promise(resolve => {
            // Expose a function that the page can call when the button is clicked
            page.exposeFunction('notifyLoginButtonClicked', () => {
              console.log('Login button clicked detected!');
              resolve();
            });
          });
          
          // Update the button click handler in the page to call our exposed function
          await page.evaluate(() => {
            // Find all login buttons and update their onclick handlers
            const loginButtons = document.querySelectorAll('.login-helper[style*="green"]');
            loginButtons.forEach(button => {
              const originalOnClick = button.onclick;
              button.onclick = function() {
                // Call the original onclick function
                if (originalOnClick) originalOnClick.call(this);
                
                // Notify that the button was clicked
                window.notifyLoginButtonClicked();
              };
            });
          });
          
          console.log('Waiting for the green button to be clicked...');
          
          // Wait for both the button click and navigation to the watch history page
          await Promise.all([
            loginButtonClickedPromise,
            page.waitForNavigation({
              timeout: 300000,
              waitUntil: 'networkidle2',
              url: url => url.includes('watch-history')
            })
          ]);
          
          console.log('Login button clicked and navigation to watch history page detected. Login completed successfully.');
        } catch (error) {
          console.error('Timed out waiting for login. Please try again.');
          throw new Error('Login timeout: Please try running the script again and complete the login within 5 minutes.');
        }
      }
    } else {
      // Navigate to Amazon login
      console.log('Navigating to Amazon login...');
      await page.goto(AMAZON_LOGIN_URL, { waitUntil: 'networkidle2' });
      
      // Take a screenshot of the login page for debugging
      console.log('Taking screenshot of the login page...');
      await page.screenshot({ path: 'login-page.png' });
    
    // Print the current URL to help diagnose redirects
    const currentUrl = page.url();
    console.log(`Current URL: ${currentUrl}`);
    
    // Print page title to help identify the page
    const pageTitle = await page.title();
    console.log(`Page title: ${pageTitle}`);
    
    // Login to Amazon
    console.log('Logging in to Amazon...');
    
    // Wait for the login page to load and check which selectors are available
    console.log('Detecting login form structure...');
    
    try {
      // Log all input fields on the page for debugging
      const inputFields = await page.evaluate(() => {
        const inputs = Array.from(document.querySelectorAll('input'));
        return inputs.map(input => ({
          type: input.type,
          id: input.id,
          name: input.name,
          placeholder: input.placeholder
        }));
      });
      
      console.log('Input fields found on page:', JSON.stringify(inputFields, null, 2));
      
      // Log all buttons on the page for debugging
      const buttons = await page.evaluate(() => {
        const btns = Array.from(document.querySelectorAll('button, input[type="submit"]'));
        return btns.map(btn => ({
          type: btn.type,
          id: btn.id,
          name: btn.name,
          text: btn.textContent || btn.value
        }));
      });
      
      console.log('Buttons found on page:', JSON.stringify(buttons, null, 2));
      
      // Check if we're on the email input page
      const emailSelector = await Promise.race([
        page.waitForSelector('#ap_email', { timeout: 5000 }).then(() => '#ap_email'),
        page.waitForSelector('input[type="email"]', { timeout: 5000 }).then(() => 'input[type="email"]'),
        page.waitForSelector('input[name="email"]', { timeout: 5000 }).then(() => 'input[name="email"]'),
        new Promise(resolve => setTimeout(() => resolve(null), 5000))
      ]);
      
      if (!emailSelector) {
        // Try to detect if we're already logged in
        const alreadyLoggedIn = await page.evaluate(() => {
          return document.body.textContent.includes('Hello,') ||
                 document.body.textContent.includes('Account') ||
                 document.body.textContent.includes('Sign Out');
        });
        
        if (alreadyLoggedIn) {
          console.log('It appears you are already logged in to Amazon. Proceeding...');
          // Skip the login process
        } else {
          console.log('Could not find email input field. Taking a full page screenshot for debugging...');
          await page.screenshot({ path: 'login-error-full.png', fullPage: true });
          
          // Try to navigate directly to Prime Video
          console.log('Attempting to navigate directly to Prime Video...');
          await page.goto('https://www.primevideo.com/', { waitUntil: 'networkidle2' });
          
          // Check if we're logged in to Prime Video
          const primeVideoLoggedIn = await page.evaluate(() => {
            return document.body.textContent.includes('Hello,') ||
                   document.body.textContent.includes('Account') ||
                   document.body.textContent.includes('Sign Out');
          });
          
          if (primeVideoLoggedIn) {
            console.log('Successfully accessed Prime Video. Proceeding...');
          } else {
            throw new Error('Could not find email input field and direct navigation to Prime Video failed. Please check login-error-full.png for details.');
          }
        }
      } else {
        console.log(`Found email input with selector: ${emailSelector}`);
        await page.type(emailSelector, config.amazon.email);
        
        // Look for the continue button
        const continueSelector = await Promise.race([
          page.waitForSelector('#continue', { timeout: 2000 }).then(() => '#continue'),
          page.waitForSelector('input[type="submit"]', { timeout: 2000 }).then(() => 'input[type="submit"]'),
          page.waitForSelector('button[type="submit"]', { timeout: 2000 }).then(() => 'button[type="submit"]'),
          page.waitForSelector('#signInSubmit', { timeout: 2000 }).then(() => '#signInSubmit'),
          new Promise(resolve => setTimeout(() => resolve(null), 2000))
        ]);
        
        if (!continueSelector) {
          throw new Error('Could not find continue button. Amazon login page structure may have changed.');
        }
        
        console.log(`Found continue button with selector: ${continueSelector}`);
        await page.click(continueSelector);
        
        // Wait for password field
        const passwordSelector = await Promise.race([
          page.waitForSelector('#ap_password', { timeout: 5000 }).then(() => '#ap_password'),
          page.waitForSelector('input[type="password"]', { timeout: 5000 }).then(() => 'input[type="password"]'),
          page.waitForSelector('input[name="password"]', { timeout: 5000 }).then(() => 'input[name="password"]'),
          new Promise(resolve => setTimeout(() => resolve(null), 5000))
        ]);
        
        if (!passwordSelector) {
          throw new Error('Could not find password input field. Amazon login page structure may have changed.');
        }
        
        console.log(`Found password input with selector: ${passwordSelector}`);
        await page.type(passwordSelector, config.amazon.password);
        
        // Look for the sign in button
        const signInSelector = await Promise.race([
          page.waitForSelector('#signInSubmit', { timeout: 2000 }).then(() => '#signInSubmit'),
          page.waitForSelector('input[type="submit"]', { timeout: 2000 }).then(() => 'input[type="submit"]'),
          page.waitForSelector('button[type="submit"]', { timeout: 2000 }).then(() => 'button[type="submit"]'),
          new Promise(resolve => setTimeout(() => resolve(null), 2000))
        ]);
        
        if (!signInSelector) {
          throw new Error('Could not find sign in button. Amazon login page structure may have changed.');
        }
        
        console.log(`Found sign in button with selector: ${signInSelector}`);
        await page.click(signInSelector);
      }
      
    } catch (error) {
      console.error('Error during login process:', error.message);
      console.log('Taking screenshot of the login page for debugging...');
      await page.screenshot({ path: 'login-error.png' });
      throw error;
      }
      
      // Check for 2FA
      try {
      // Wait for potential 2FA screen
      await page.waitForSelector('#auth-mfa-otpcode, .cvf-widget-input-code', { timeout: 5000 });
      console.log('2FA detected! Please check your device for the verification code');
      
      // Wait for user to enter the code manually
      console.log('Please enter the 2FA code in the browser window...');
      
      // Wait for navigation after 2FA
      await page.waitForNavigation({ timeout: 60000, waitUntil: 'networkidle2' });
      console.log('2FA completed successfully');
    } catch (error) {
      // No 2FA prompt appeared, continue normally
      console.log('No 2FA prompt detected, continuing...');
      }
      
      // Wait for login to complete
      await page.waitForNavigation({ waitUntil: 'networkidle2' });
      
      // Navigate to Prime Video watch history
      console.log('Navigating to Prime Video watch history...');
      await page.goto(PRIME_VIDEO_WATCH_HISTORY_URL, { waitUntil: 'networkidle2' });
    }
    
    // Make sure we're on the watch history page with improved error handling
    console.log('Verifying we are on the watch history page...');
    
    try {
      // Check current URL
      const currentUrl = page.url();
      console.log(`Current URL: ${currentUrl}`);
      
      const isWatchHistoryPage = currentUrl.includes('watch-history');
      
      if (!isWatchHistoryPage) {
        console.log('Not on watch history page. Attempting to navigate to watch history...');
        
        // Wait a bit before navigation to ensure page stability
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        try {
          // Try a simpler URL first
          const simpleWatchHistoryUrl = 'https://www.primevideo.com/settings/watch-history';
          console.log(`Navigating to ${simpleWatchHistoryUrl}...`);
          
          // Use a longer timeout and more relaxed waitUntil condition
          await page.goto(simpleWatchHistoryUrl, {
            timeout: 60000,
            waitUntil: 'domcontentloaded'
          });
          
          // Wait for the page to stabilize
          await new Promise(resolve => setTimeout(resolve, 5000));
          
          // Check if navigation was successful
          const newUrl = page.url();
          console.log(`New URL after navigation: ${newUrl}`);
          
          if (!newUrl.includes('watch-history')) {
            console.log('Navigation to simple URL failed. Trying alternative approach...');
            
            // Try clicking a link or using browser history instead of direct navigation
            const navigationSuccess = await page.evaluate(() => {
              // Try to find any link to watch history on the page
              const watchHistoryLinks = Array.from(document.querySelectorAll('a'))
                .filter(a => a.href && a.href.includes('watch-history'));
              
              if (watchHistoryLinks.length > 0) {
                console.log('Found watch history link, clicking it...');
                watchHistoryLinks[0].click();
                return true;
              }
              
              return false;
            });
            
            if (navigationSuccess) {
              // Wait for navigation to complete
              await page.waitForNavigation({ timeout: 30000 }).catch(e => {
                console.warn('Navigation timeout after clicking link:', e.message);
              });
              
              // Wait for the page to stabilize
              await new Promise(resolve => setTimeout(resolve, 5000));
            } else {
              console.log('Could not find watch history links. Using browser history API...');
              
              // Try using browser history API
              await page.evaluate(() => {
                window.location.href = 'https://www.primevideo.com/settings/watch-history';
              });
              
              // Wait for navigation to complete
              await new Promise(resolve => setTimeout(resolve, 5000));
            }
          }
        } catch (navigationError) {
          console.warn('Error during navigation to watch history:', navigationError.message);
          console.log('Attempting to continue despite navigation error...');
        }
      } else {
        console.log('Already on watch history page.');
      }
    } catch (verificationError) {
      console.warn('Error verifying watch history page:', verificationError.message);
    }
    
    // Take a screenshot of the current page regardless of navigation success
    try {
      await page.screenshot({ path: 'current-page.png' });
      console.log('Screenshot saved as current-page.png');
    } catch (screenshotError) {
      console.warn('Error taking screenshot:', screenshotError.message);
    }
    
    // Scroll to load all watch history with improved error handling
    console.log('Loading complete watch history...');
    let previousHeight;
    let currentHeight = 0;
    let scrollAttempts = 0;
    let noChangeCount = 0;
    const maxScrollAttempts = 1000; // Extremely high limit to ensure we load everything
    const maxNoChangeCount = 10; // Number of times height can remain unchanged before considering complete
    
    try {
      console.log('Implementing simple, brute-force scrolling to load all items...');
      
      // First, get initial height
      currentHeight = await page.evaluate(() => document.body.scrollHeight);
      console.log(`Initial page height: ${currentHeight}`);
      
      // Simple brute-force scrolling loop
      while ((previousHeight !== currentHeight || noChangeCount < maxNoChangeCount) && scrollAttempts < maxScrollAttempts) {
        if (previousHeight === currentHeight) {
          noChangeCount++;
          console.log(`No height change detected (${noChangeCount}/${maxNoChangeCount})`);
        } else {
          noChangeCount = 0; // Reset counter when height changes
        }
        
        previousHeight = currentHeight;
        scrollAttempts++;
        
        try {
          // Simple scroll to bottom
          console.log(`Scroll attempt ${scrollAttempts}/${maxScrollAttempts}`);
          
          // First check if we're still on the watch history page
          const isOnWatchHistoryPage = await page.evaluate(() => {
            return window.location.href.includes('watch-history');
          }).catch(() => false);
          
          if (!isOnWatchHistoryPage) {
            console.log('Not on watch history page. Attempting to navigate back...');
            throw new Error('Navigation detected');
          }
          
          // Scroll to bottom - simple and direct
          await page.evaluate(() => {
            window.scrollTo(0, document.body.scrollHeight);
          });
          
          // Wait a long time for content to load (15 seconds)
          console.log('Waiting 15 seconds for content to load...');
          await new Promise(resolve => setTimeout(resolve, 15000));
          
          // Every 5 attempts, try to find and click "Load More" buttons
          if (scrollAttempts % 5 === 0) {
            console.log('Checking for "Load More" buttons...');
            try {
              const clickedButton = await page.evaluate(() => {
                // Common selectors for load more buttons
                const loadMoreSelectors = [
                  'button[data-automation-id*="load-more"]',
                  'button:contains("Load More")',
                  'button:contains("Show More")',
                  'a:contains("Load More")',
                  'a:contains("Show More")',
                  'div[role="button"]:contains("Load More")',
                  'div[role="button"]:contains("Show More")',
                  '.load-more',
                  '.show-more',
                  '[data-testid*="load-more"]',
                  '[data-testid*="show-more"]',
                  '[class*="loadMore"]',
                  '[class*="showMore"]'
                ];
                
                // Try each selector
                for (const selector of loadMoreSelectors) {
                  try {
                    const loadMoreElements = document.querySelectorAll(selector);
                    for (const element of loadMoreElements) {
                      // Check if the element is visible
                      const rect = element.getBoundingClientRect();
                      const isVisible = rect.top >= 0 && rect.left >= 0 &&
                                       rect.bottom <= window.innerHeight &&
                                       rect.right <= window.innerWidth;
                      
                      if (isVisible && element.offsetParent !== null) {
                        console.log('Found and clicking "Load More" button');
                        element.click();
                        return true; // Exit after clicking one button
                      }
                    }
                  } catch (e) {
                    // Ignore errors for individual selectors
                  }
                }
                return false;
              });
              
              if (clickedButton) {
                console.log('Clicked a "Load More" button, waiting for content to load...');
                await new Promise(resolve => setTimeout(resolve, 10000)); // Wait longer after clicking
              }
            } catch (buttonError) {
              console.warn('Error checking for load more buttons:', buttonError.message);
            }
          }
          
          // Get new height
          currentHeight = await page.evaluate(() => document.body.scrollHeight);
          console.log(`Scroll attempt ${scrollAttempts}: Previous height: ${previousHeight}, Current height: ${currentHeight}`);
          
          // Every 10 attempts, count items to show progress
          if (scrollAttempts % 10 === 0) {
            try {
              const itemCount = await page.evaluate(() => {
                // Count all date sections
                const dateSections = document.querySelectorAll('div[data-automation-id=activity-history-items] > ul > li');
                let totalItems = 0;
                
                // Count items in each date section
                dateSections.forEach(section => {
                  const items = section.querySelector('ul')?.querySelectorAll('li') || [];
                  totalItems += items.length;
                });
                
                return {
                  dateSections: dateSections.length,
                  totalItems: totalItems
                };
              });
              
              console.log(`Current progress: ${itemCount.dateSections} date sections, ${itemCount.totalItems} total items`);
              
              // If we have 300+ items, we can stop
              if (itemCount.totalItems >= 300) {
                console.log(`Found ${itemCount.totalItems} items, which is >= 300. Stopping scrolling.`);
                break;
              }
            } catch (countError) {
              console.warn('Error counting items:', countError.message);
            }
          }
        } catch (scrollError) {
          console.warn(`Error during scrolling attempt ${scrollAttempts}:`, scrollError.message);
          
          // Check if this is an execution context error
          const isContextError = scrollError.message.includes('Execution context was destroyed') ||
                                scrollError.message.includes('context') ||
                                scrollError.message.includes('navigation');
          
          if (isContextError) {
            console.log('Detected execution context error due to navigation. Implementing recovery strategy...');
          }
          
          // Implement a more robust recovery strategy
          try {
            // First, wait a bit to let any navigation complete
            await new Promise(resolve => setTimeout(resolve, 3000));
            
            // Check if we're still on a valid page
            try {
              const url = await page.url();
              console.log(`Current URL after error: ${url}`);
            } catch (urlError) {
              console.warn('Could not get URL, page might be in an invalid state:', urlError.message);
            }
            
            // Use a simpler URL and more relaxed navigation options
            const simpleWatchHistoryUrl = 'https://www.primevideo.com/settings/watch-history';
            console.log(`Attempting recovery from scrolling error: Navigating to ${simpleWatchHistoryUrl}...`);
            
            // Use a longer timeout and more relaxed waitUntil condition
            await page.goto(simpleWatchHistoryUrl, {
              timeout: 60000,
              waitUntil: 'domcontentloaded'
            });
            
            // Wait longer for the page to stabilize
            await new Promise(resolve => setTimeout(resolve, 5000));
            
            // Check if navigation was successful
            const newUrl = page.url();
            console.log(`Recovery navigation result: ${newUrl}`);
            
            if (!newUrl.includes('watch-history')) {
              console.log('Recovery navigation failed. Trying alternative approach...');
              
              // Try using browser history API as a last resort
              await page.evaluate(() => {
                window.location.href = 'https://www.primevideo.com/settings/watch-history';
              }).catch(e => {
                console.warn('Error using browser history API:', e.message);
              });
              
              // Wait for navigation to complete
              await new Promise(resolve => setTimeout(resolve, 5000));
            }
            
            // Get new height after recovery with error handling
            try {
              currentHeight = await page.evaluate(() => document.body.scrollHeight);
              console.log(`New height after recovery: ${currentHeight}`);
            } catch (heightError) {
              console.warn(`Could not get height after recovery: ${heightError.message}`);
              // If we can't get the height, use a default value to continue
              currentHeight = previousHeight + 1; // Force loop to continue
            }
          } catch (recoveryError) {
            console.error('Failed to recover from scrolling error:', recoveryError.message);
            console.log('Will attempt to continue with extraction despite recovery failure');
            
            // Force the loop to continue by changing the height
            currentHeight = previousHeight + 1;
            
            // If we've had too many consecutive errors, break out of the loop
            if (scrollAttempts > maxScrollAttempts / 2) {
              console.log('Too many scroll errors, proceeding with extraction anyway');
              break;
            }
          }
        }
      }
      
      // Final item count
      try {
        const finalCount = await page.evaluate(() => {
          // Count all date sections
          const dateSections = document.querySelectorAll('div[data-automation-id=activity-history-items] > ul > li');
          let totalItems = 0;
          
          // Count items in each date section
          dateSections.forEach(section => {
            const items = section.querySelector('ul')?.querySelectorAll('li') || [];
            totalItems += items.length;
          });
          
          return {
            dateSections: dateSections.length,
            totalItems: totalItems
          };
        });
        
        console.log(`Scrolling complete. Final stats - Date sections: ${finalCount.dateSections}, Items: ${finalCount.totalItems}`);
        
        if (scrollAttempts >= maxScrollAttempts) {
          console.log(`Reached maximum scroll attempts (${maxScrollAttempts}). Proceeding with extraction.`);
        }
      } catch (finalCountError) {
        console.warn('Error getting final item count:', finalCountError.message);
      }
      
      if (scrollAttempts >= maxScrollAttempts) {
        console.log(`Reached maximum scroll attempts (${maxScrollAttempts}). Proceeding with extraction.`);
      }
    } catch (error) {
      console.warn('Error during scrolling process:', error.message);
      console.log('Attempting to continue with extraction anyway...');
    }
    
    // Extract watch history with improved error handling
    console.log('Extracting watch history...');
    let watchHistory = [];
    
    try {
      // Take a screenshot before extraction
      await page.screenshot({ path: 'before-extraction.png' });
      
      // Make sure we're still on the watch history page
      const currentUrl = page.url();
      if (!currentUrl.includes('watch-history')) {
        console.log('Not on watch history page before extraction. Navigating back...');
        // Use a simpler URL and more relaxed navigation options
        const simpleWatchHistoryUrl = 'https://www.primevideo.com/settings/watch-history';
        console.log(`Navigating to ${simpleWatchHistoryUrl} before extraction...`);
        
        // Use a longer timeout and more relaxed waitUntil condition
        await page.goto(simpleWatchHistoryUrl, {
          timeout: 60000,
          waitUntil: 'domcontentloaded'
        });
        
        // Wait longer for the page to stabilize
        await new Promise(resolve => setTimeout(resolve, 5000));
      }
      
      // Extract in smaller chunks to avoid context destruction
      // First, get the number of date sections
      const dateCount = await page.evaluate(() => {
        const dateSections = document.querySelectorAll('div[data-automation-id=activity-history-items] > ul > li');
        return dateSections.length;
      });
      
      console.log(`Found ${dateCount} date sections. Extracting in chunks...`);
      
      // Process each date section individually
      for (let i = 0; i < dateCount; i++) {
        try {
          const sectionResults = await page.evaluate((sectionIndex, useOriginalTitles) => {
            const results = [];
            
            // Get the specific date section
            const dateSections = document.querySelectorAll('div[data-automation-id=activity-history-items] > ul > li');
            const dateSection = dateSections[sectionIndex];
            
            if (!dateSection) return results;
            
            const dateWatchedString = dateSection.querySelector('[data-automation-id^="wh-date"]')?.textContent || 'Unknown Date';
            const mediaSections = dateSection.querySelector('ul')?.querySelectorAll('li') || [];
            
            // Loop over media watched for this date
            for (const mediaSection of mediaSections) {
              // Try to get both localized and original title
              const localizedTitle = mediaSection.querySelector('img')?.alt || 'Unknown Title';
              
              // Try different ways to get the original title
              let originalTitle = '';
              
              // Check for data attributes that might contain the original title
              const imgElement = mediaSection.querySelector('img');
              if (imgElement) {
                // Check various data attributes that might contain the original title
                originalTitle = imgElement.getAttribute('data-original-title') ||
                               imgElement.getAttribute('data-title') ||
                               imgElement.getAttribute('title') ||
                               imgElement.dataset.originalTitle ||
                               '';
                
                // If no data attribute, try to find it in other elements
                if (!originalTitle) {
                  // Look for elements that might contain the original title
                  const possibleTitleElements = mediaSection.querySelectorAll('.original-title, [data-automation-id*="title"], .title');
                  for (const element of possibleTitleElements) {
                    if (element.textContent && element.textContent.trim() !== localizedTitle) {
                      originalTitle = element.textContent.trim();
                      break;
                    }
                  }
                }
              }
              
              // Use original title if available and option is enabled, otherwise use localized title
              const title = (useOriginalTitles && originalTitle) ? originalTitle : localizedTitle;
              const episodesWatchedCheckbox = mediaSection.querySelector('[type="checkbox"]');
              
              // If the 'Episodes watched' checkbox exists, it's a series
              if (episodesWatchedCheckbox) {
                // Click the checkbox to load episodes if not already checked
                if (!episodesWatchedCheckbox.checked) {
                  episodesWatchedCheckbox.click();
                }
                
                // Get episodes immediately (no setTimeout)
                const episodeSections = mediaSection.querySelectorAll('[data-automation-id^=wh-episode] > div > p');
                
                // If no episodes are found, add the show without episode info
                if (!episodeSections || episodeSections.length === 0) {
                  results.push({
                    date: dateWatchedString,
                    title: title,
                    episode: null
                  });
                } else {
                  // Loop over episodes
                  for (const episodeSection of episodeSections) {
                    const episodeTitle = episodeSection?.textContent?.trim() || 'Unknown Episode';
                    results.push({
                      date: dateWatchedString,
                      title: title,
                      episode: episodeTitle
                    });
                  }
                }
              } else {
                // It's a movie
                results.push({
                  date: dateWatchedString,
                  title: title,
                  episode: null
                });
              }
            }
            
            return results;
          }, i, config.useOriginalTitles);
          
          // Add results from this section to the main array
          watchHistory = watchHistory.concat(sectionResults);
          console.log(`Extracted ${sectionResults.length} items from date section ${i+1}/${dateCount}`);
          
          // Small delay between sections to avoid overloading the page
          await new Promise(resolve => setTimeout(resolve, 500));
          
        } catch (sectionError) {
          console.warn(`Error extracting date section ${i+1}/${dateCount}:`, sectionError.message);
          // Continue with the next section
        }
      }
    } catch (extractionError) {
      console.error('Error during watch history extraction:', extractionError.message);
      // Take a screenshot to help diagnose the issue
      try {
        await page.screenshot({ path: 'extraction-error.png' });
      } catch (screenshotError) {
        console.warn('Could not take error screenshot:', screenshotError.message);
      }
    }
    
    console.log(`Found ${watchHistory.length} items in watch history`);
    
    // Process each item to get metadata
    console.log('Processing items and fetching metadata...');
    const processedItems = [];
    
    for (const item of watchHistory) {
      const processedItem = await processItem(item.date, item.title, item.episode);
      processedItems.push(processedItem);
      console.log(`Processed: ${item.title} ${item.episode ? `- ${item.episode}` : ''}`);
    }
    
    // Generate CSV
    console.log('Generating CSV...');
    const header = [
      'simkl_id', 'TVDB_ID', 'TMDB', 'IMDB_ID', 'MAL_ID',
      'Type', 'Title', 'Year', 'LastEpWatched', 'Watchlist',
      'WatchedDate', 'Rating', 'Memo'
    ];
    
    const csvData = [header, ...processedItems];
    
    // Write to file
    stringify(csvData, (err, output) => {
      if (err) throw err;
      fs.writeFileSync(config.outputPath, output);
      console.log(`CSV file saved to ${config.outputPath}`);
    });
    
    console.log('Export completed successfully!');
  } catch (error) {
    console.error('Error during export:', error);
    console.log('\nTROUBLESHOOTING TIPS:');
    console.log('1. Check the screenshots (login-page.png, login-error.png, login-error-full.png) for details');
    console.log('2. Try clearing your browser cookies or using a different browser profile');
    console.log('3. If you want to attempt automatic login, run with:');
    console.log('   ATTEMPT_AUTO_LOGIN=true npm start');
    console.log('4. Ensure your Amazon credentials in config.js are correct if using automatic login');
  } finally {
    await browser.close();
  }
}

// Run the script
scrapeWatchHistory();

// Export functions for testing
export {
  lookupMetadata,
  parseDate,
  processItem,
  imdbClient,
  malClient,
  simklClient,
  tmdbClient,
  tvdbClient
};