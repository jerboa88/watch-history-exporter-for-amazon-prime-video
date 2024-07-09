/* 
	Watch History Exporter for Amazon Prime Video | johng.io | Public Domain
	Export your Amazon Prime Video watch history as a CSV file.
	
	Instructions: 
		1. Open https://www.primevideo.com/settings/watch-history in your browser
		2. Scroll to the bottom of the page to force load all items
		3. Copy this script into the devtools console and run it
*/


// Parse the watch history and return an array of arrays
const parseWatchHistory = () => {
	// Initialize an empty array to store the watch history
	const watchHistoryArray = [];

	// Select all list items within the watch history
	const watchHistoryItems = document.querySelectorAll('div[data-automation-id=activity-history-items] > ul > li');

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
	}

	return watchHistoryArray;
};


// Force lazy loading of the watch history by scrolling to the bottom of the page
const forceLoadWatchHistory = async () => {
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
	const mimeTypePrefix = 'data:text/csv;charset=utf-8,';
	const headers = ['Date Watched', 'Type', 'Title', 'Episode'];
	const csvContent = `${mimeTypePrefix}${headers.join(', ')}\n${inputArray.map(e => e.join(', ')).join('\n')}`;

	window.open(encodeURI(csvContent));
};


// Entry point
const main = async () => {
	await forceLoadWatchHistory();
	downloadCsv(parseWatchHistory());
};


main();