<!-- Project Header -->
<div align="center">
  <h1 class="projectName">Watch History Exporter for Amazon Prime Video</h1>

  <p class="projectBadges">
    <img src="https://img.shields.io/badge/type-JS_Script-4caf50.svg" alt="Project type" title="Project type">
    <img src="https://img.shields.io/github/languages/top/jerboa88/watch-history-exporter-for-amazon-prime-video.svg" alt="Language" title="Language">
    <img src="https://img.shields.io/github/repo-size/jerboa88/watch-history-exporter-for-amazon-prime-video.svg" alt="Repository size" title="Repository size">
    <a href="LICENSE">
      <img src="https://img.shields.io/github/license/jerboa88/watch-history-exporter-for-amazon-prime-video.svg" alt="Project license" title="Project license"/>
    </a>
  </p>

  <p class="projectDesc" data-exposition="A script for exporting your Amazon Prime Video watch history as a CSV file.">
    A script to export your Amazon Prime Video watch history as a CSV file.
  </p>

  <br/>
</div>


## About
This script runs in your browser and allows you to save your watch history from [Amazon Prime Video] to a CSV file, where it can be processed further or imported into other platforms.

## Usage
You can run the script by copying the code in [watch-history-exporter-for-amazon-prime-video.js] and pasting it into your browser's devtools console.

> [!CAUTION]
> For security reasons, I do not recommend running scripts from the internet unless you understand what they are doing. If you are not a developer, I recommend reading the comments in the code and/or asking a LLM like [ChatGPT] to explain it to you.

**Detailed steps:**
 1. Open [primevideo.com/settings/watch-history] in your browser
 2. Open your browser's devtools console ([how?])
 3. Copy the code in [watch-history-exporter-for-amazon-prime-video.js] and paste it into the console. If this doesn't work or you see a warning message about pasting, see the [FAQ].
 4. Press enter to run the script. You should see the script running in the console and you'll be prompted to save a file when it finishes. If this doesn't happen, see the [FAQ].

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

Contributions and forks are welcome. By contributing code, you agree to waive all claim of copyright to your work and release it to the public domain.


## License
This project is released into the public domain. See the [LICENSE] for details. Attribution is appreciated but not required :)


[watch-history-exporter-for-amazon-prime-video.js]: watch-history-exporter-for-amazon-prime-video.js
[LICENSE]: LICENSE
[FAQ]: #FAQ
[create an issue]: https://github.com/jerboa88/watch-history-exporter-for-amazon-prime-video/issues
[primevideo.com/settings/watch-history]: https://www.primevideo.com/settings/watch-history
[Amazon Prime Video]: https://www.primevideo.com
[this video]: https://youtu.be/X5uyCtVD1-o?si=AOrzgez90KiDlA-z&t=11
[Chrome for Developers Blog]: https://developer.chrome.com/blog/self-xss
[ChatGPT]: https://chatgpt.com/
[how?]: https://balsamiq.com/support/faqs/browserconsole/
