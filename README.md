## **Font-Ass_Tool**

#### A simple small utility to mux a subtitle file with its font dependencies into a Matroska-Container

___

### **Installation:**

Make sure you have the following software already installed:
+ fontconfig
+ ffmpeg
+ cargo + rust (for compiling)

```
cargo install --path .
export PATH="$HOME/.cargo/bin:$PATH"
fa_tool check subtitle.ass
fa_tool run subtitle.ass
```
___

### **Notes:**

This script parses the subtitle file, asks fontconfig for a matching font and muxes them into one file.
+ The mkv output is not a playable file, it's only made for an easier remuxing progress. (MPV options like `external-file` are not supported)
    * This affects all FFmpeg based applications. If your subtitle file doesn't start at time 0, it won't play as expected. (MPV will always skip to the beginning of the first track)
    * Hint: put a video track into the container, or play the subtitle file externally
+ FFmpeg is run with `overwrite` disabled by default. 

___

### **_Problems_:**

I got angry, learned C++ and searched GitHub. After a while I found that Aegisub's font collector almost perfectly matches all font names you feed into it, so I decided to translate parts of it's `font_file_lister_fontconfig.cpp` file to rust. We thus also finally have "warnings" when a font couldn't be found. (indicated by the font path = `Not found`).

~~This code should be able to handle all correctly formatted ASS subtitle files.~~ 

If there is anyone who knows how to get a correct match for fonts like `Bahnschrift`, then please create an issue, or reach out to me on Discord!


___
