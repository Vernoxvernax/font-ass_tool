## **Font-Ass_Tool**

#### A simple small utility to mux a subtitle file with its font dependencies into a Matroska-Container

___

### **Installation:**

#### **Linux:**
___

Make sure you have the following software already installed:
+ fontconfig
+ FFmpeg
+ cargo + rust

```
cargo install --path .
export PATH="$HOME/.cargo/bin:$PATH"
```


#### **Windows (by compiling and linking to fontconfig):**
___

Make sure you have some version of Visual Studio installed (tested with 2022 Community).

Additional requirements:
+ meson
+ ninja
+ git
+ FFmpeg
+ cargo + rust (for compiling)
+ brain

Read and run `build_fontconfig.bat`.

Now run `cargo build --release`.

Read and run `copy_dlls.bat` (unless you want to add three different folders to your path).

Any tips on how to improve the `dll` situation are very welcome.

#### **Dlopen:**
___

I don't really know (or care) what this feature is supposed to be useful for, but it requires `pkg-config`. So pretty painful to set up on Windows (MSYS).

### **Usage:**

```
fa_tool check subtitle.ass
fa_tool run subtitle.ass
```

Replace `fa_tool` with `cargo run --release --` when on Windows.

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
