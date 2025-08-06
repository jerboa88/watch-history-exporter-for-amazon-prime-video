/**
 * Prime Video to Simkl CSV Exporter (Browser Version)
 *
 * IMPORTANT: This script requires a config.js file in the same directory.
 * Create a config.js file based on config.template.js with your API keys:
 *
 * export default {
 *   simklClientId: 'YOUR_SIMKL_CLIENT_ID',
 *   simklClientSecret: 'YOUR_SIMKL_SECRET',
 *   tmdbApiKey: 'YOUR_TMDB_API_KEY',
 *   tvdbApiKey: 'YOUR_TVDB_API_KEY',
 *   imdbApiKey: 'YOUR_IMDB_API_KEY',
 *   malClientId: 'YOUR_MAL_CLIENT_ID',
 *   priorityOrder: ['simkl', 'tmdb', 'tvdb', 'imdb', 'mal'],
 *   rateLimit: {
 *     simkl: { calls: 30, perSeconds: 10 },
 *     tmdb: { calls: 40, perSeconds: 10 },
 *     tvdb: { calls: 100, perSeconds: 60 },
 *     imdb: { calls: 100, perSeconds: 60 },
 *     mal: { calls: 2, perSeconds: 1 }
 *   }
 * };
 */

// Try to load config
let CONFIG;
try {
  if (typeof require !== 'undefined') {
    CONFIG = require('./config.js');
  } else if (typeof window !== 'undefined' && window.CONFIG) {
    CONFIG = window.CONFIG;
  } else {
    throw new Error('Config not found');
  }
} catch (e) {
  console.warn('Config file not found. Please create a config.js file based on config.template.js');
  CONFIG = {
    simklClientId: '',
    simklClientSecret: '',
    tmdbApiKey: '',
    tvdbApiKey: '',
    imdbApiKey: '',
    malClientId: '',
    priorityOrder: ['simkl', 'tmdb', 'tvdb', 'imdb', 'mal'],
    rateLimit: {
      simkl: { calls: 30, perSeconds: 10 },
      tmdb: { calls: 40, perSeconds: 10 },
      tvdb: { calls: 100, perSeconds: 60 },
      imdb: { calls: 100, perSeconds: 60 },
      mal: { calls: 2, perSeconds: 1 }
    }
  };
}

(async () => {

	// Configuration for metadata lookup
	const METADATA_CONFIG = {
		priorityOrder: ['simkl', 'tmdb', 'tvdb', 'imdb', 'mal'],
		cache: new Map(),
		rateLimit: {
			simkl: { calls: 30, perSeconds: 10 },
			tmdb: { calls: 40, perSeconds: 10 },
			tvdb: { calls: 100, perSeconds: 60 },
			imdb: { calls: 100, perSeconds: 60 },
			mal: { calls: 2, perSeconds: 1 }
		}
	};

	// API Client for Simkl
	const simklClient = {
		apiKey: null,
		baseUrl: 'https://api.simkl.com',

		init: function(clientId, clientSecret) {
			this.apiKey = clientId;
			this.apiSecret = clientSecret;
		},

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
					log(`Simkl rate limit hit, retrying after ${retryAfter} seconds`, console.warn);
					await new Promise(resolve => setTimeout(resolve, retryAfter * 1000));
					return this.search(title, type, year);
				}

				if (!response.ok) {
					throw new Error(`Simkl API error: ${response.statusText}`);
				}

				const data = await response.json();
				return data || [];
			} catch (error) {
				log(`Simkl search failed: ${error.message}`, console.warn);
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

	// API Client for TMDB
	const tmdbClient = {
		apiKey: null,
		baseUrl: 'https://api.themoviedb.org/3',

		init: function(apiKey) {
			this.apiKey = apiKey;
		},

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

	// API Client for TVDB
	const tvdbClient = {
		apiKey: null,
		baseUrl: 'https://api.thetvdb.com',
		token: null,

		init: function(apiKey) {
			this.apiKey = apiKey;
		},

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

	// API Client for IMDB
	const imdbClient = {
		apiKey: null,
		baseUrl: 'https://imdb-api.com/en/API',

		init: function(apiKey) {
			this.apiKey = apiKey;
		},

		search: async function(title, type) {
			if (!this.apiKey) {
				console.warn('IMDB API key not configured');
				return { results: [] };
			}

			try {
				const endpoint = type === 'movie' ? 'SearchMovie' : 'SearchSeries';
				const response = await fetch(`${this.baseUrl}/${endpoint}/${this.apiKey}/${encodeURIComponent(title)}`);

				if (response.status === 429) {
					log(`IMDB API rate limit hit, waiting before retry`, console.warn);
					await new Promise(resolve => setTimeout(resolve, 5000));
					return this.search(title, type);
				}

				if (!response.ok) {
					throw new Error(`IMDB API error: ${response.statusText}`);
				}

				const data = await response.json();
				return data;
			} catch (error) {
				log(`IMDB search failed: ${error.message}`, console.warn);
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

	// API Client for MyAnimeList
	const malClient = {
		clientId: null,
		baseUrl: 'https://api.myanimelist.net/v2',

		init: function(clientId) {
			this.clientId = clientId;
		},

		search: async function(title) {
			if (!this.clientId) {
				console.warn('MyAnimeList Client ID not configured');
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
					log(`MyAnimeList API rate limit hit, waiting before retry`, console.warn);
					await new Promise(resolve => setTimeout(resolve, 5000));
					return this.search(title);
				}

				if (!response.ok) {
					throw new Error(`MyAnimeList API error: ${response.statusText}`);
				}

				const data = await response.json();
				return data;
			} catch (error) {
				log(`MyAnimeList search failed: ${error.message}`, console.warn);
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
		
		// Check cache first
		if (METADATA_CONFIG.cache.has(cacheKey)) {
			return METADATA_CONFIG.cache.get(cacheKey);
		}

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
		for (const api of METADATA_CONFIG.priorityOrder) {
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
							if (tmdbSearch.results.length > 0) {
								const tmdbData = await tmdbClient.getIds(tmdbSearch.results[0].id, type);
								result.TMDB = tmdbData.id;
								result.IMDB_ID = tmdbData.imdb_id || '';
							}
						}
						break;
					case 'tvdb':
						if (!result.TVDB_ID) {
							const tvdbSearch = await tvdbClient.search(title, type, year);
							if (tvdbSearch.data.length > 0) {
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

		// Cache the result
		METADATA_CONFIG.cache.set(cacheKey, result);
		return result;
	};

	// Delimiters for the CSV file
	const DELIMITER = {
		string: '"',
		field: ',',
		record: '\n',
	};

	const I18N_COMMON_ES = {
		date_watched: 'Fecha vista',
		episode_title: 'Episodio',
		movie: 'Película',
		series: 'Serie',
		title: 'Título',
		type: 'Tipo',
		parseDateString: (dateString) =>
			parseDateString(
				dateString,
				// ex. 23 de abril de 2024
				/(?<d>\d{1,2}) de (?<m>[a-zA-Z]+) de (?<y>\d{4})/,
			),
	};

	const I18N_COMMON_PT = {
		date_watched: 'Data assistida',
		episode_title: 'Episódio',
		movie: 'Filme',
		series: 'Série',
		title: 'Título',
		type: 'Tipo',
		parseDateString: (dateString) =>
			parseDateString(
				dateString,
				// ex. 23 de Abril de 2024
				/(?<d>\d{1,2}) de (?<m>[a-zA-Zç]+) de (?<y>\d{4})/,
			),
	};

	const I18N_COMMON_ZH = {
		date_watched: '觀看日期',
		episode_title: '集',
		movie: '電影',
		series: '劇集系列',
		title: '標題',
		type: '類型',
		parseDateString: (dateString) =>
			parseDateString(
				dateString,
				// ex. 2024年4月23日
				/(?<y>\d{4})年(?<m>\d{1,2})月(?<d>\d{1,2})日/,
				true,
			),
	};

	// Locale-specific strings and functions
	const I18N = {
		'de-de': {
			date_watched: 'Datum angesehen',
			episode_title: 'Folge',
			movie: 'Film',
			series: 'Serie',
			title: 'Titel',
			type: 'Typ',
			parseDateString: (dateString) =>
				// ex. 23. April 2024
				parseDateString(
					dateString,
					/(?<d>\d{1,2})\. (?<m>[a-zA-Zä]+) (?<y>\d{4})/,
				),
		},
		'en-us': {
			date_watched: 'Date Watched',
			episode_title: 'Episode',
			movie: 'Movie',
			series: 'Series',
			title: 'Title',
			type: 'Type',
			// ex. April 23, 2024
			parseDateString: (dateString) => new Date(dateString),
		},
		'es-419': I18N_COMMON_ES,
		'es-es': {
			...I18N_COMMON_ES,
			date_watched: 'Fecha de visualización',
		},
		'fr-fr': {
			date_watched: 'Date regardée',
			episode_title: 'Épisode',
			movie: 'Film',
			series: 'Série',
			title: 'Titre',
			type: 'Type',
			parseDateString: (dateString) =>
				parseDateString(
					dateString,
					// ex. 23 avril 2024
					/(?<d>\d{1,2}) (?<m>[a-zA-Zéû]+) (?<y>\d{4})/,
				),
		},
		'pt-br': I18N_COMMON_PT,
		'pt-pt': {
			...I18N_COMMON_PT,
			date_watched: 'Data de visualização',
		},
		'zh-cn': {
			...I18N_COMMON_ZH,
			series: '剧集系列',
		},
		'zh-tw': I18N_COMMON_ZH,
		'ja-jp': {
			date_watched: '視聴日',
			episode_title: 'エピソード',
			movie: '映画',
			series: 'シリーズ',
			title: 'タイトル',
			type: '種類',
			// ex. 2024/04/23
			parseDateString: (dateString) => new Date(dateString),
		},
	};

	// Print an informational message to the console
	const log = (msg, logFn = console.info, showPrefix = true) => {
		const prefixArray = showPrefix
			? [
					'%c[Watch History Exporter for Amazon Prime]',
					'color:#1399FF;background:#00050d;font-weight:bold;',
				]
			: [];

		logFn(...prefixArray, msg);
	};

	// Get a list of long month names for a given language
	// Based on code by Maksim (https://dev.to/pretaporter/how-to-get-month-list-in-your-language-4lfb)
	function getMonthNames(languageTag) {
		const formatter = new Intl.DateTimeFormat(languageTag, { month: 'long' });

		return Object.fromEntries(
			[...Array(12).keys()]
				.map((monthIndex) => formatter.format(new Date(2025, monthIndex)))
				// Convert to lowercase to avoid case sensitivity issues
				.map((key, index) => [key.toLowerCase(), index]),
		);
	}

	// Parse a localized date string to a Date object
	const parseDateString = (dateString, regex, isMonthNumeric = false) => {
		const { y, m, d } = regex.exec(dateString).groups;

		return new Date(
			Number.parseInt(y),
			isMonthNumeric
				? Number.parseInt(m) - 1
				: i18n.monthNames[m.toLowerCase()],
			Number.parseInt(d),
		);
	};

	// Convert a localized date string to an ISO date string
	const toIsoDateString = (dateString) => {
		const date = i18n.parseDateString(dateString);

		if (Number.isNaN(date.getTime())) {
			console.groupEnd();
			console.groupEnd();
			console.groupEnd();
			log(
				'Unsupported date format. Try changing the language of your Amazon Prime Video account to English',
				console.error,
			);
			throw new Error();
		}

		return date.toISOString().split('T')[0];
	};

	// Add a movie or episode to the array
	const addItem = async (
		watchHistoryArray,
		dateWatchedString,
		title,
		episodeTitle,
	) => {
		const isoDateWatchedString = toIsoDateString(dateWatchedString);
		const mediaType = episodeTitle ? 'tv' : 'movie';
		const yearMatch = title.match(/\((\d{4})\)/);
		const year = yearMatch ? yearMatch[1] : '';
		const cleanTitle = title.replace(/\s*\(\d{4}\)$/, '');

		// Get metadata for the title
		const metadata = await lookupMetadata(cleanTitle, mediaType, year);

		// Format episode number if available
		let lastEpWatched = '';
		if (episodeTitle) {
			const epMatch = episodeTitle.match(/(\d+)/);
			if (epMatch) {
				lastEpWatched = `s1e${epMatch[1]}`;
			}
		}

		watchHistoryArray.push([
			metadata.simkl_id || '',
			metadata.TVDB_ID || '',
			metadata.TMDB || '',
			metadata.IMDB_ID || '',
			metadata.MAL_ID || '',
			mediaType,
			`${DELIMITER.string}${cleanTitle}${DELIMITER.string}`,
			year,
			lastEpWatched,
			'completed',
			isoDateWatchedString,
			'', // Rating
			''  // Memo
		]);

		return watchHistoryArray;
	};

	// Parse the watch history and return an array of arrays
	const parseWatchHistory = async () => {
		log('Parsing watch history... Items found:', console.group);

		// Initialize an empty array to store the watch history
		const watchHistoryArray = [];

		// Select all list items within the watch history
		const dateSections = document.querySelectorAll(
			'div[data-automation-id=activity-history-items] > ul > li',
		);

		// Loop over date sections
		for (const dateSection of dateSections) {
			const mediaSections = dateSection.querySelectorAll('& > ul > li');
			const dateWatchedString = dateSection.querySelector(
				'[data-automation-id^="wh-date"]',
			).textContent;

			log(dateWatchedString, console.group, false);

			// Loop over media watched for each date
			for (const mediaSection of mediaSections) {
				const episodesWatchedCheckbox =
					mediaSection.querySelector('[type="checkbox"]');
				const title = mediaSection.querySelector('img').alt;

				// If the 'Episodes watched' checkbox exists, it's a series
				// Otherwise, it's a movie
				if (episodesWatchedCheckbox) {
					log(`[${i18n.series}] ${title}`, console.group, false);

					// Click the 'Episodes watched' checkbox if it exists to get the episode information
					if (!episodesWatchedCheckbox.checked) {
						// A click event is required to load the episode information (checking from DOM doesn't work)
						episodesWatchedCheckbox.click();
					}

					const episodeSections = mediaSection.querySelectorAll(
						'[data-automation-id^=wh-episode] > div > p',
					);

					// Loop over episodes watched for each series
					for (const episodeSection of episodeSections) {
						const episodeTitle = episodeSection?.textContent?.trim();

						log(episodeTitle, console.info, false);
						await addItem(watchHistoryArray, dateWatchedString, title, episodeTitle);
					}

					console.groupEnd();
				} else {
					log(`[${i18n.movie}] ${title}`, console.info, false);
					await addItem(watchHistoryArray, dateWatchedString, title);
				}
			}

			console.groupEnd();
		}

		console.groupEnd();

		return watchHistoryArray;
	};

	// Force lazy loading of the watch history by scrolling to the bottom of the page
	const forceLoadWatchHistory = async () => {
		log('Loading watch history...');

		return new Promise((resolve) => {
			const autoScrollInterval = setInterval(() => {
				if (
					!document.querySelector(
						'div[data-automation-id=activity-history-items] > div > noscript',
					)
				) {
					clearInterval(autoScrollInterval);
					resolve();
				}

				window.scrollTo(0, document.body.scrollHeight);
			}, 500);
		});
	};

	// Download the watch history as a CSV file
	const downloadCsv = (inputArray) => {
		log('Saving CSV file...', console.group);
		log(
			'If you are not prompted to save a file, make sure "Pop-ups and redirects" and "Automatic downloads" are enabled for www.primevideo.com in your browser.',
			console.info,
			false,
		);
		console.groupEnd();

		const columnNames = [
			'simkl_id',
			'TVDB_ID',
			'TMDB',
			'IMDB_ID',
			'MAL_ID',
			'Type',
			'Title',
			'Year',
			'LastEpWatched',
			'Watchlist',
			'WatchedDate',
			'Rating',
			'Memo'
		];
		const csvData = [columnNames, ...inputArray]
			.map((item) => item.join(DELIMITER.field))
			.join(DELIMITER.record);
		const csvDataUrl = `data:text/csv;charset=utf-8,${encodeURIComponent(csvData)}`;

		window.open(csvDataUrl);
	};

	// Script entry point
	log('Script started');
	
	// Verify required configuration
	if (!CONFIG.simklClientId || CONFIG.simklClientId === 'YOUR_SIMKL_CLIENT_ID') {
		log('Simkl Client ID is required in CONFIG. Script cannot continue.', console.error);
		return;
	}
	
	// Initialize API clients
	simklClient.init(CONFIG.simklClientId);
	if (CONFIG.tmdbApiKey && CONFIG.tmdbApiKey !== 'YOUR_TMDB_API_KEY') {
		tmdbClient.init(CONFIG.tmdbApiKey);
	}
	if (CONFIG.tvdbApiKey && CONFIG.tvdbApiKey !== 'YOUR_TVDB_API_KEY') {
		tvdbClient.init(CONFIG.tvdbApiKey);
	}
	if (CONFIG.imdbApiKey && CONFIG.imdbApiKey !== 'YOUR_IMDB_API_KEY') {
		imdbClient.init(CONFIG.imdbApiKey);
	}
	if (CONFIG.malClientId && CONFIG.malClientId !== 'YOUR_MAL_CLIENT_ID') {
		malClient.init(CONFIG.malClientId);
	}
	
	const languageTag = document.documentElement.lang;
	let i18n = I18N[languageTag];

	if (!i18n) {
		log(
			`Language "${languageTag}" is not supported. The script may fail`,
			console.warn,
		);

		i18n = I18N['en-us'];
	}

	i18n.monthNames = getMonthNames(languageTag);

	await forceLoadWatchHistory();
	const watchHistory = await parseWatchHistory();
	downloadCsv(watchHistory);
	log('Script finished');
})() && 'Script loaded';
