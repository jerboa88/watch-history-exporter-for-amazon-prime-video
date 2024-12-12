(() => {
	// Delimiters for the CSV file
	const DELIMITER = {
		STRING: '"',
		FIELD: ',',
		RECORD: '\n',
	};

	//  Column names for the CSV file
	const COLUMN_NAME = {
		DATE_WATCHED: 'Date Watched',
		TYPE: 'Type',
		TITLE: 'Title',
		EPISODE_TITLE: 'Episode',
	};

	//  Values for the TYPE column in the CSV file
	const MEDIA_TYPE_NAME = {
		SERIES: 'Series',
		MOVIE: 'Movie',
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

	// Add a movie or episode to the array
	const addItem = (watchHistoryArray, dateWatched, title, episodeTitle) => {
		const formattedDateWatched = new Date(dateWatched)
			.toISOString()
			.split('T')[0];
		const mediaType = episodeTitle
			? MEDIA_TYPE_NAME.SERIES
			: MEDIA_TYPE_NAME.MOVIE;
		const formattedTitle = `${DELIMITER.STRING}${title}${DELIMITER.STRING}`;
		const formattedEpisodeTitle = episodeTitle
			? `${DELIMITER.STRING}${episodeTitle}${DELIMITER.STRING}`
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
			const mediaSections = dateSection.querySelectorAll('ul > li');
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
					log(`[${MEDIA_TYPE_NAME.SERIES}] ${title}`, false, true);

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
					log(`[${MEDIA_TYPE_NAME.MOVIE}] ${title}`, false);
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

		const mimeTypeString = 'data:text/csv;charset=utf-8,';
		const columnNamesString = Object.values(COLUMN_NAME).join(DELIMITER.FIELD);
		const dataRowsString = inputArray
			.map((item) => item.join(DELIMITER.FIELD))
			.join(DELIMITER.RECORD);
		const csvContent = `${mimeTypeString}${columnNamesString}${DELIMITER.RECORD}${dataRowsString}`;

		window.open(encodeURI(csvContent));
	};

	// Entry point
	const main = async () => {
		log('Script started');
		await forceLoadWatchHistory();
		downloadCsv(parseWatchHistory());
		log('Script finished');
	};

	return main();
})();
