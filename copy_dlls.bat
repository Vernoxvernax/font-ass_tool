@echo off

IF exist .\fontconfig\build\src (
IF exist .\target\release\. (
echo "Copying dll's to the release target folder (./target/release/.)"
copy .\fontconfig\build\src\fontconfig-1.dll .\target\release\. /y
copy .\fontconfig\build\subprojects\freetype2\freetype-6.dll .\target\release\. /y
cd .\fontconfig\build\subprojects\expat*
copy expat.dll ..\..\..\..\target\release\. /y
cd ..\..\..\..\.
pause
) ELSE (
echo Please create .\target\release\. or run cargo build --release.
pause
)
) ELSE (
echo Please run build_fontconfig.bat.
pause
)

:: This gotta be one of the worst batch script on the internet lol


