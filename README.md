## **Font-Ass_Tool**

#### A simple small utility to mux a subtitle file with it's font dependencies into a matroska container

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
+ The mkv output is not a playable file, it's only made for an easier remuxing progress. (mpv options like `external-file` are not supported)
    * This affects all ffmpeg based applications. If your subtitle file doesn't start at time 0, this will lead to playback issues. (mpv will always skip to the beginning of the first track)
    * Hint: mux a video track into the container or play the subtitle file externally
+ ffmpeg is run with `overwrite` disabled, but without any bindings. Because of constant linking problems I decided to just run ffmpeg commands so just make sure ffmpeg is in your PATH.

___

### **~~Problems~~:**

I got angry, learned C++ and searched Github. After a while I found that Aegisub's font collector almost (liability) perfectly matches all font names you feed into it, so decided to translate parts of it's `font_file_lister_fontconfig.cpp` file to rust, which finally also gives me the option to add "warnings" when a font has not been found. (indicated by the font path = `Not found`). This code should be able to handle all correctly formated ASS subtitle files.

Have fun.

___
