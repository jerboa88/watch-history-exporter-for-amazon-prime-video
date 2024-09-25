/*
	Watch History Exporter for Amazon Prime Video | johng.io | Public Domain
	Export your Amazon Prime Video watch history as a CSV file.

	Instructions:
		1. Open https://www.primevideo.com/settings/watch-history in your browser
		2. Copy this script into the devtools console and run it

	Notes:
		- Some browsers prevent you from pasting code in the console because it could be malicious. If this happens, follow the instructions in the console to re-enable pasting
		- If you are not prompted to save a file, make sure "Pop-ups and redirects" and "Automatic downloads" are enabled for www.primevideo.com in your browser
*/


(() => {
	// Print an informational message to the console
	const log = (msg, showPrefix = true, startGroup = false) => {
		const logFunc = startGroup ? console.group : console.info;
		const prefixArray = showPrefix ? ['%c[Watch History Exporter for Amazon Prime]', 'color:#1399FF;background:#00050d;font-weight:bold;'] : [];

		logFunc(...prefixArray, msg);
	};

	// Parse the watch history and return an array of arrays
	const parseWatchHistory = () => {
		log('Parsing watch history...', true, true);

		// Initialize an empty array to store the watch history
		const watchHistoryArray = [];

		// Select all list items within the watch history
		const watchHistoryItems = document.querySelectorAll('div[data-automation-id=activity-history-items] > ul > li');

		log('Items found:', false, true);

		for (const item of watchHistoryItems) {
			const itemDetails = item.querySelector('ul > li');
			const episodesWatchedCheckbox = itemDetails.querySelector('[type="checkbox"]');
			let itemType = 'Movie';

			// Click the 'Episodes watched' checkbox if it exists to get the episode information
			if (episodesWatchedCheckbox) {
				itemType = 'Series';

				// A click event is required to load the episode information (checking from DOM doesn't work)
				if (!episodesWatchedCheckbox.checked) {
					episodesWatchedCheckbox.click();
				}
			}

			// Extract information and print to the console
			const dateWatched = item.querySelector('[data-automation-id^="wh-date"]').textContent;
			const title = itemDetails.querySelector('img').alt;
			const episodeInfo = itemDetails.querySelector('[data-automation-id^=wh-episode] > div > p');

			watchHistoryArray.push([
				new Date(dateWatched).toISOString().split('T')[0],
				itemType,
				`"${title}"`,
				episodeInfo ? `"${episodeInfo.textContent.trim()}"` : ''
			]);

			log(`[${itemType}] ${title} `, false);
		}

		console.groupEnd();

		return watchHistoryArray;
	};


	// Force lazy loading of the watch history by scrolling to the bottom of the page
	const forceLoadWatchHistory = async () => {
		log('Loading watch history...');

		return new Promise((resolve) => {
			const autoScrollInterval = setInterval(() => {
				if (!document.querySelector("div[data-automation-id=activity-history-items] > div > noscript")) {
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
		log('If you are not prompted to save a file, make sure "Pop-ups and redirects" and "Automatic downloads" are enabled for www.primevideo.com in your browser.', false)
		console.groupEnd();

		const mimeTypePrefix = 'data:text/csv;charset=utf-8,';
		const headers = ['Date Watched', 'Type', 'Title', 'Episode'];
		const csvContent = `${mimeTypePrefix}${headers.join(', ')}\n${inputArray.map(e => e.join(', ')).join('\n')}`;

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