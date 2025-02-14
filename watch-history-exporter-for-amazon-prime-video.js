(async () => {
	// Delimiters for the CSV file
	const DELIMITER = {
		string: '"',
		field: ',',
		record: '\n',
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
					/(?<d>\d{1,2})\. (?<m>[a-zA-ZäöüÄÖÜß]+) (?<y>\d{4})/,
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
				.map((key, index) => [key, index]),
		);
	}

	// Parse a localized date string to a Date object
	const parseDateString = (dateString, regex, isMonthNumeric = false) => {
		const { y, m, d } = regex.exec(dateString).groups;

		return new Date(
			Number.parseInt(y),
			isMonthNumeric ? Number.parseInt(m - 1) : i18n.monthNames[m],
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
				true,
				console.error,
			);
			throw new Error();
		}

		return date.toISOString().split('T')[0];
	};

	// Add a movie or episode to the array
	const addItem = (
		watchHistoryArray,
		dateWatchedString,
		title,
		episodeTitle,
	) => {
		const isoDateWatchedString = toIsoDateString(dateWatchedString);
		const mediaType = episodeTitle ? i18n.series : i18n.movie;
		const formattedTitle = `${DELIMITER.string}${title}${DELIMITER.string}`;
		const formattedEpisodeTitle = episodeTitle
			? `${DELIMITER.string}${episodeTitle}${DELIMITER.string}`
			: '';

		watchHistoryArray.push([
			isoDateWatchedString,
			mediaType,
			formattedTitle,
			formattedEpisodeTitle,
		]);

		return watchHistoryArray;
	};

	// Parse the watch history and return an array of arrays
	const parseWatchHistory = () => {
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
						addItem(watchHistoryArray, dateWatchedString, title, episodeTitle);
					}

					console.groupEnd();
				} else {
					log(`[${i18n.movie}] ${title}`, console.info, false);
					addItem(watchHistoryArray, dateWatchedString, title);
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
			i18n.date_watched,
			i18n.type,
			i18n.title,
			i18n.episode_title,
		];
		const csvData = [columnNames, ...inputArray]
			.map((item) => item.join(DELIMITER.field))
			.join(DELIMITER.record);
		const csvDataUrl = `data:text/csv;charset=utf-8,${encodeURIComponent(csvData)}`;

		window.open(csvDataUrl);
	};

	// Script entry point
	log('Script started');
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
	downloadCsv(parseWatchHistory());
	log('Script finished');
})() && 'Script loaded';
