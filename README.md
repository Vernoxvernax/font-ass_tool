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
fa_tool run subtitle.ass
```
___

### **Notes**:

+ This programm does **NOT** check whether the required fonts are installed.
It only parses the subtitle file, asks fontconfig for a matching font and muxes them into one file.
+ The mkv output is not a playable file, it's only made for an easier remuxing progress. !mpv options like `external-file` are not supported!
    * This also affects mpv. If your subtitle file doesn't start at time 0, this will lead to playback issues. (mpv will always skips to the beginning of the file)
    * Hint: mux a video track into the container or play the subtitle file externally
+ ffmpeg is run with `overwrite` disabled
