(async () => {
	// Delimiters for the CSV file
	const DELIMITER = {
		string: '"',
		field: ',',
		record: '\n',
	};

	// Translated strings
	const I18N = {
		'en-us': {
			date_watched: 'Date Watched',
			episode_title: 'Episode',
			movie: 'Movie',
			series: 'Series',
			title: 'Title',
			type: 'Type',
		},
	};

	// Print an informational message to the console
	const log = (msg, showPrefix = true, startGroup = false) => {
		const logFunc = startGroup ? console.group : console.info;
		const prefixArray = showPrefix
			? [
					'%c[Watch History Exporter for Amazon Prime]',
					'color:#1399FF;background:#00050d;font-weight:bold;',
				]
			: [];

		logFunc(...prefixArray, msg);
	};

	// Parse an English date string (e.g. "December 14, 2021") into a Date object
	const englishDateToISO = (englishDateString) => new Date(englishDateString);

	// Parse a German date string (e.g. "14. Dezember 2021") into a Date object
	const germanDateToISO = (germanDate) => {
		const months = {
			Januar: 0,
			Februar: 1,
			März: 2,
			April: 3,
			Mai: 4,
			Juni: 5,
			Juli: 6,
			August: 7,
			September: 8,
			Oktober: 9,
			November: 10,
			Dezember: 11,
		};

		const dateParts = germanDate.match(
			/^(\d{1,2})\. ([A-Za-zäöüÄÖÜß]+) (\d{4})$/,
		);

		if (!dateParts) throw new Error('Invalid German date format');

		const day = Number.parseInt(dateParts[1], 10);
		const month = months[dateParts[2]];
		const year = Number.parseInt(dateParts[3], 10);

		if (month === undefined) throw new Error('Invalid German month name');

		const date = new Date(year, month, day);

		return date;
	};

	// Convert a localized date string to an ISO date string
	const toIsoDateString = (dateString) => {
		const locale = document.documentElement.lang;
		const date = {
			'de-de': germanDateToISO,
			'en-us': englishDateToISO,
		}[locale](dateString);

		if (!date) {
			throw new Error(
				'Invalid date format. Try changing the language of your Amazon Prime Video account to English',
			);
		}

		return date.toISOString().split('T')[0];
	};

	// Add a movie or episode to the array
	const addItem = (watchHistoryArray, dateWatched, title, episodeTitle) => {
		const formattedDateWatched = toIsoDateString(dateWatched);
		const mediaType = episodeTitle ? i18n.series : i18n.movie;
		const formattedTitle = `${DELIMITER.string}${title}${DELIMITER.string}`;
		const formattedEpisodeTitle = episodeTitle
			? `${DELIMITER.string}${episodeTitle}${DELIMITER.string}`
			: '';

		watchHistoryArray.push([
			formattedDateWatched,
			mediaType,
			formattedTitle,
			formattedEpisodeTitle,
		]);

		return watchHistoryArray;
	};

	// Parse the watch history and return an array of arrays
	const parseWatchHistory = () => {
		log('Parsing watch history... Items found:', true, true);

		// Initialize an empty array to store the watch history
		const watchHistoryArray = [];

		// Select all list items within the watch history
		const dateSections = document.querySelectorAll(
			'div[data-automation-id=activity-history-items] > ul > li',
		);

		// Loop over date sections
		for (const dateSection of dateSections) {
			const mediaSections = dateSection.querySelectorAll('& > ul > li');
			const dateWatched = dateSection.querySelector(
				'[data-automation-id^="wh-date"]',
			).textContent;

			log(dateWatched, false, true);

			// Loop over media watched for each date
			for (const mediaSection of mediaSections) {
				const episodesWatchedCheckbox =
					mediaSection.querySelector('[type="checkbox"]');
				const title = mediaSection.querySelector('img').alt;

				// If the 'Episodes watched' checkbox exists, it's a series
				// Otherwise, it's a movie
				if (episodesWatchedCheckbox) {
					log(`[${i18n.series}] ${title}`, false, true);

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

						log(episodeTitle, false);
						addItem(watchHistoryArray, dateWatched, title, episodeTitle);
					}

					console.groupEnd();
				} else {
					log(`[${i18n.movie}] ${title}`, false);
					addItem(watchHistoryArray, dateWatched, title);
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
		log('Saving CSV file...', true, true);
		log(
			'If you are not prompted to save a file, make sure "Pop-ups and redirects" and "Automatic downloads" are enabled for www.primevideo.com in your browser.',
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

	const i18n = I18N['en-us'];

	await forceLoadWatchHistory();
	downloadCsv(parseWatchHistory());
	log('Script finished');
})();
