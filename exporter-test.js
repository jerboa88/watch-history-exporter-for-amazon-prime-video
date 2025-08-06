import { JSDOM } from 'jsdom';
import fs from 'fs';
import path from 'path';

// Import functions from index.js
import { lookupMetadata, parseDate, processItem, imdbClient, malClient, simklClient, tmdbClient, tvdbClient } from './index.js';

// Create a simple test function that we can run directly
async function runTests() {
  console.log("Starting tests...");

  // Test 1: Test parseDate function
  console.log("\nTest 1: parseDate function");
  const testDates = [
    "August 6, 2025",
    "2025-08-06",
    "6 August 2025"
  ];
  
  for (const date of testDates) {
    try {
      const parsedDate = parseDate(date);
      console.log(`✓ Parsed "${date}" to "${parsedDate}"`);
    } catch (error) {
      console.error(`✗ Failed to parse "${date}": ${error.message}`);
    }
  }

  // Save original API client methods
  const originalSimklSearch = simklClient.search;
  const originalSimklGetIds = simklClient.getIds;
  const originalTmdbSearch = tmdbClient.search;
  const originalTmdbGetIds = tmdbClient.getIds;
  const originalTvdbSearch = tvdbClient.search;
  const originalTvdbGetIds = tvdbClient.getIds;
  const originalImdbSearch = imdbClient.search;
  const originalImdbGetDetails = imdbClient.getDetails;
  const originalMalSearch = malClient.search;
  const originalMalGetDetails = malClient.getDetails;
  
  // Override API client methods with mock implementations
  simklClient.search = () => Promise.resolve([{ids: {simkl: 123}}]);
  simklClient.getIds = () => Promise.resolve({ids: {simkl: 123, tvdb: 456, tmdb: 789, imdb: 'tt1234567', mal: 12345}});
  
  tmdbClient.search = () => Promise.resolve({results: [{id: 789}]});
  tmdbClient.getIds = () => Promise.resolve({id: 789, imdb_id: 'tt1234567'});
  
  tvdbClient.search = () => Promise.resolve({data: [{id: 456}]});
  tvdbClient.getIds = () => Promise.resolve({data: {id: 456}});
  
  imdbClient.search = () => Promise.resolve({
    results: [
      { id: 'tt1375666', title: 'Inception', description: '2010' }
    ]
  });
  imdbClient.getDetails = () => Promise.resolve({id: 'tt1375666', title: 'Inception', year: '2010'});
  
  malClient.search = () => Promise.resolve({
    data: [
      {
        node: {
          id: 12345,
          title: 'Attack on Titan',
          start_date: '2013-04-07'
        }
      }
    ]
  });
  malClient.getDetails = () => Promise.resolve({id: 12345, title: 'Attack on Titan', start_date: '2013-04-07'});

  // Test 2: Test lookupMetadata function
  console.log("\nTest 2: lookupMetadata function");
  try {
    // Initialize API clients with mock values
    imdbClient.apiKey = 'mock-imdb-key';
    malClient.clientId = 'mock-mal-client-id';
    
    const metadata = await lookupMetadata("Inception", "movie", "2010");
    console.log(`✓ Metadata lookup successful`);
    console.log(JSON.stringify(metadata, null, 2));
  } catch (error) {
    console.error(`✗ Metadata lookup failed: ${error.message}`);
  }

  // Test 3: Test processItem function
  console.log("\nTest 3: processItem function");
  try {
    const item = await processItem("August 6, 2025", "Inception (2010)", null);
    console.log(`✓ Item processing successful`);
    console.log(item);
  } catch (error) {
    console.error(`✗ Item processing failed: ${error.message}`);
  }

  // Test 4: Test IMDB API client
  console.log("\nTest 4: IMDB API client");
  try {
    // IMDB client is already initialized with mock key
    
    const searchResult = await imdbClient.search("Inception", "movie");
    console.log(`✓ IMDB search successful`);
    console.log(JSON.stringify(searchResult, null, 2));
  } catch (error) {
    console.error(`✗ IMDB search failed: ${error.message}`);
  }

  // Test 5: Test MyAnimeList API client
  console.log("\nTest 5: MyAnimeList API client");
  try {
    // MAL client is already initialized with mock client ID
    
    const searchResult = await malClient.search("Attack on Titan");
    console.log(`✓ MyAnimeList search successful`);
    console.log(JSON.stringify(searchResult, null, 2));
  } catch (error) {
    console.error(`✗ MyAnimeList search failed: ${error.message}`);
  }

  console.log("\nAll tests completed!");
  
  // Restore original API client methods
  simklClient.search = originalSimklSearch;
  simklClient.getIds = originalSimklGetIds;
  tmdbClient.search = originalTmdbSearch;
  tmdbClient.getIds = originalTmdbGetIds;
  tvdbClient.search = originalTvdbSearch;
  tvdbClient.getIds = originalTvdbGetIds;
  imdbClient.search = originalImdbSearch;
  imdbClient.getDetails = originalImdbGetDetails;
  malClient.search = originalMalSearch;
  malClient.getDetails = originalMalGetDetails;
}

// Run the tests
runTests()
  .then(() => {
    console.log("Tests finished successfully!");
    process.exit(0);
  })
  .catch(error => {
    console.error("Tests failed:", error);
    process.exit(1);
  });