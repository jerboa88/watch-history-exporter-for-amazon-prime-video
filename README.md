<!-- Project Header -->
<div align="center">
  <img class="projectLogo" src="screenshot.png" alt="Project logo" title="Project logo" width="512">

  <h1 class="projectName">Watch History Exporter for Amazon Prime Video</h1>

  <p class="projectBadges">
    <img src="https://johng.io/badges/category/Script.svg" alt="Project category" title="Project category">
    <img src="https://img.shields.io/github/languages/top/jerboa88/watch-history-exporter-for-amazon-prime-video.svg" alt="Language" title="Language">
    <img src="https://img.shields.io/github/repo-size/jerboa88/watch-history-exporter-for-amazon-prime-video.svg" alt="Repository size" title="Repository size">
    <a href="LICENSE">
      <img src="https://img.shields.io/github/license/jerboa88/watch-history-exporter-for-amazon-prime-video.svg" alt="Project license" title="Project license"/>
    </a>
  </p>

  <p class="projectDesc">
    A script to export your Amazon Prime Video watch history as a CSV file.
  </p>

  <br/>
</div>


## About
This script runs in your browser and allows you to save your watch history from [Amazon Prime Video] to a CSV file, where it can be processed further or imported into other platforms.

### Features
- **⚡ Browser-based:** Run the script directly in your browser, no installation required
- **📥 Detailed export:** Save your complete watch history as a CSV file with the following columns:
  - Date Watched
  - Type (movie or TV show)
  - Title
  - Episode Title (for TV shows)
- **🌍 Multi-language support:** Built-in support for the following languages:
  - Deutsch
  - English
  - Español
  - Español Latinoamérica
  - Français
  - Português (Brasil)
  - Português (Portugal)
  - 日本語
  - 简体中文
  - 繁體中文


## Usage
> [!CAUTION]
> For security reasons, I do not recommend running scripts from the internet unless you understand what they are doing. If you are not a developer, I recommend reading the comments in the code and/or asking a LLM like [ChatGPT] to explain it to you.
>

You can run the script by going to [primevideo.com/settings/watch-history], copying the code in [watch-history-exporter-for-amazon-prime-video.js], and pasting it into your browser's devtools console.

<details>
  <summary><b>Detailed instructions:</b></summary>
  <ol>
    <li>Open <a href="https://www.primevideo.com/settings/watch-history">primevideo.com/settings/watch-history</a> in your browser</li>
    <li>Open your browser's devtools console (<a href="https://balsamiq.com/support/faqs/browserconsole/">how?</a>)</li>
    <li>Copy the code in <a href="watch-history-exporter-for-amazon-prime-video.js">watch-history-exporter-for-amazon-prime-video.js</a> and paste it into the console. If this doesn't work or you see a warning message about pasting, see the <a href="#FAQ">FAQ</a>.</li>
    <li>Press enter to run the script. You should see the script running in the console and you'll be prompted to save a file when it finishes. If this doesn't happen, see the <a href="#FAQ">FAQ</a>.</li>
  </ol>
</details>


## FAQ

### Nothing shows up when I paste in the console / I get a warning when I try to paste in the console
Some browsers prevent you from pasting code in the console because it could be malicious. This is called Paste Protection and you can read more about it on the [Chrome for Developers Blog].

If this happens, follow the instructions in the console to re-enable pasting, and then try again. For Chrome, the following steps should work:
 1. Try to paste something in the console. You should get a warning message about pasting
 2. Type "allow pasting" in the console and press enter

 See [this video] for a visual walkthrough.

### I get an `Uncaught SyntaxError: Unexpected identifier` error when running the script
Make sure that you select the entire file with <kbd>Ctrl</kbd> + <kbd>A</kbd> when copying it. If part of the script is cut off, it won't work.

### The script runs, but I am not prompted to save a file
If you have a default download folder set, check if the file is there.

Otherwise, make sure "Pop-ups and redirects" and "Automatic downloads" are enabled for www.primevideo.com in your browser settings.


## Contributing
If you encounter any problems with the script, feel free to [create an issue].

Pull requests, bug reports, translations, and other kinds of contributions are greatly appreciated. By contributing code, you agree to waive all claim of copyright to your work and release it to the public domain.


## License
This project is released into the public domain. See the [LICENSE] for details. Attribution is appreciated but not required :)


## 💕 Funding

Find this project useful? [Sponsoring me](https://johng.io/funding) will help me cover costs and **_commit_** more time to open-source.

If you can't donate but still want to contribute, don't worry. There are many other ways to help out, like:

- 📢 reporting (submitting feature requests & bug reports)
- 👨‍💻 coding (implementing features & fixing bugs)
- 📝 writing (documenting & translating)
- 💬 spreading the word
- ⭐ starring the project

I appreciate the support!


[watch-history-exporter-for-amazon-prime-video.js]: watch-history-exporter-for-amazon-prime-video.js
[LICENSE]: LICENSE
[create an issue]: https://github.com/jerboa88/watch-history-exporter-for-amazon-prime-video/issues
[primevideo.com/settings/watch-history]: https://www.primevideo.com/settings/watch-history
[Amazon Prime Video]: https://www.primevideo.com
[this video]: https://youtu.be/X5uyCtVD1-o?si=AOrzgez90KiDlA-z&t=11
[Chrome for Developers Blog]: https://developer.chrome.com/blog/self-xss
[ChatGPT]: https://chatgpt.com/
