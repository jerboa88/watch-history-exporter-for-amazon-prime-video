// Configuration for Prime Video to Simkl CSV Exporter
export default {
  // API Keys
  simklClientId: 'YOUR_SIMKL_CLIENT_ID',
  simklClientSecret: 'YOUR_SIMKL_SECRET',
  tmdbApiKey: 'YOUR_TMDB_API_KEY',
  tvdbApiKey: 'YOUR_TVDB_API_KEY',
  // No IMDB API key needed - using free imdbapi.dev API
  malClientId: 'YOUR_MAL_CLIENT_ID',
  malClientSecret: 'YOUR_MAL_CLIENT_SECRET',
  
  // Amazon Login Credentials
  amazon: {
    email: 'YOUR_AMAZON_EMAIL',
    password: 'YOUR_AMAZON_PASSWORD'
  },
  
  // Output Settings
  outputPath: './export.csv',
  
  // Metadata Settings
  priorityOrder: ['simkl', 'tmdb', 'tvdb', 'imdb', 'mal'],
  useOriginalTitles: true, // Set to true to use original titles instead of localized titles
  
  // Rate Limiting
  rateLimit: {
    simkl: { calls: 30, perSeconds: 10 },
    tmdb: { calls: 40, perSeconds: 10 },
    tvdb: { calls: 100, perSeconds: 60 },
    imdb: { calls: 5, perSeconds: 10 }, // imdbapi.dev has stricter rate limits
    mal: { calls: 2, perSeconds: 1 }
  }
};