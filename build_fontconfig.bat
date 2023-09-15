@echo off

echo Downloading fontconfig and compiling it with meson & ninja

git clone "https://gitlab.freedesktop.org/fontconfig/fontconfig.git"

cmd.exe /k ""C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat" -startdir=none -arch=x64 -host_arch=x64 && cd fontconfig && meson build && ninja -C build""
pause
