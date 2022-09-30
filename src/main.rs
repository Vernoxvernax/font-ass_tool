use std::fs;
use std::path::Path;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr::{null_mut};
use std::str::from_utf8_unchecked;
use ass_deserialize::AssFont;
use clap::{Arg, Command, ArgAction};
use walkdir::WalkDir;
use fontconfig::fontconfig::{
    FcMatchPattern, FcResultMatch, FcChar8, FcBool, FcConfig,
    FcPatternFormat, FcNameParse, FcFontMatch, FcConfigSubstitute, FcDefaultSubstitute, FcConfigGetCurrent, FcPatternAddBool
};
pub mod ass_deserialize;
pub mod error;
use crate::ass_deserialize::AssFile;
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = Command::new("fa_tool")
        .about("easily batch through subtitles and its dependencies")
        .version(VERSION)
        .subcommand_required(true)
        .arg_required_else_help(true)
        .author("Vernox Vernax")
        .subcommand(
            Command::new("run")
            .short_flag('r')
            .long_flag("run")
            .about("Analyze a subtitle file written in ASS, and mux the required fonts into a matroska container.")
            .arg(
                Arg::new("file")
                    .help("list of files or folders")
                    .required(true)
                    .action(ArgAction::Set)
                    .num_args(1..)
            )
        )
        .subcommand(
            Command::new("check")
            .short_flag('c')
            .long_flag("check")
            .about("Analyze a subtitle file written in ASS, and output the best matches in the console.")
            .arg(
                Arg::new("file")
                .help("list of files or folders")
                .required(true)
                .action(ArgAction::Set)
                .num_args(1..)
            )
        )
    .get_matches();
    match matches.subcommand() {
        Some(("run", run_matches)) => {
            let args = run_matches.get_many::<String>("file");
            let files = args.unwrap().map(|s| s.to_string()).collect::<Vec<_>>();
            let raw_files = to_file_list(files);
            let ass_files = deserialize(raw_files.clone());
            for (file, name) in ass_files.iter().zip(raw_files.iter()) {
                if remux_this(find_font_files(file.clone()), name.clone()).is_err() {
                    println!("Failed to write {}:\n{:?}", name, file.fonts);
                };
            }
        },
        Some(("check", check_matches)) => {
            let args = check_matches.get_many::<String>("file");
            let files = args.unwrap().map(|s| s.to_string()).collect::<Vec<_>>();
            let raw_files = to_file_list(files);
            let ass_files = deserialize(raw_files.clone());
            for (file, name) in ass_files.iter().zip(raw_files.iter()) {
                println!("{}:", name.clone());
                for (font_file, assfont) in find_font_files(file.clone()).fonts.iter().zip(file.fonts.iter()) {
                    println!("  {}      (b: {} i: {})       => {}", assfont.facename, assfont.bold, assfont.italic , font_file.path);
                };
                println!("");
            }
        }
        _ => unreachable!(),
    }
}

fn find_font_files(file: &AssFile) -> AssFile {
    let mut fonts: Vec<AssFont> = vec![];
    let default_font = get_fallback_font();
    for font in &file.fonts {
        let mut assfont = font.clone();
        let mut font_name = font.facename.replace("@", "");
        loop {
            let cstr_font = CString::new(&*font_name).unwrap();
            unsafe {
                let pattern = FcNameParse(cstr_font.as_ptr() as *mut FcChar8);
                FcPatternAddBool(pattern, "FC_OUTLINE".as_ptr() as *const i8, true as FcBool);
                FcConfigSubstitute(null_mut(), pattern, FcMatchPattern);
                FcDefaultSubstitute(pattern);
                let mut result = 0;
                let fontmatch = FcFontMatch(null_mut(), pattern, &mut result);
                if result == FcResultMatch {
                    let oformat = CString::new("%{file}").unwrap();
                    let fonting = FcPatternFormat(fontmatch, oformat.as_ptr() as *const u8);
                    let string = CStr::from_ptr(fonting as *const c_char).to_bytes_with_nul();
                    assfont.path = from_utf8_unchecked(string).replace("\0", "").trim().to_string();
                } else {
                    panic!("No match has been found for {}", font.facename);
                };
                if assfont.path == default_font {
                    if font_name.ends_with("Condensed")
                    || font_name.ends_with("Bold")
                    || font_name.ends_with("Italic")
                    || font_name.ends_with("Semi")
                    || font_name.ends_with("Extra")
                    || font_name.ends_with("Light") {
                        font_name = font_name.trim_end_matches("Condensed").trim().to_string();
                        font_name = font_name.trim_end_matches("Bold").trim().to_string();
                        font_name = font_name.trim_end_matches("Semi").trim().to_string();
                        font_name = font_name.trim_end_matches("Italic").trim().to_string();
                        font_name = font_name.trim_end_matches("Light").trim().to_string();
                        font_name = font_name.trim_end_matches("Extra").trim().to_string();
                    } else {
                        fonts.append(&mut vec![assfont]);
                        break
                    }
                } else {
                    fonts.append(&mut vec![assfont]);
                    break
                }
            }
        }
    };
    AssFile { fonts }
}

fn get_fallback_font() -> String {
    unsafe {
        let config: *mut FcConfig = FcConfigGetCurrent();
        let default_pattern = FcNameParse("".as_ptr() as *mut FcChar8);
        FcDefaultSubstitute(default_pattern);
        FcConfigSubstitute(config, default_pattern, FcMatchPattern);
        let mut result = 0;
        let fontmatch = FcFontMatch(null_mut(), default_pattern, &mut result);
        if result == FcResultMatch {
            let oformat = CString::new("%{file}").unwrap();
            let fonting = FcPatternFormat(fontmatch, oformat.as_ptr() as *const u8);
            let string = CStr::from_ptr(fonting as *const c_char).to_bytes_with_nul();
            let default_font = from_utf8_unchecked(string).replace("\0", "").trim().to_string();
            default_font
        } else {
            "Arial".to_string()
        }
    }
}

fn remux_this(file: AssFile, name: String) -> Result<(), String> {
    if Path::new(format!("{}.mkv", name).as_str()).exists() {
        println!("{}.mkv already exists.", name);
        return Ok(());
    }
    let mut duppl_check = String::new();
    let mut cmd = "-i ".to_owned() + name.as_str();
    let mut track_index = 1;
    for assfont in file.fonts {
        if duppl_check.contains(&assfont.path) {
            continue
        }
        cmd = cmd.to_owned() + " -attach " + assfont.path.as_str();
        if assfont.path.ends_with(".ttf") {
            cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " mimetype=application/x-truetype-font";
        } else if assfont.path.ends_with(".otf") {
            cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " mimetype=application/x-font-opentype";
        } else if assfont.path.ends_with("ttc") {
            cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " mimetype=application/x-truetype-collection";
        }
        track_index += 1;
        duppl_check.push_str(&assfont.path);
    };
    cmd = cmd.to_owned() + " " + name.as_str() + ".mkv -n";
    let arg = format!("{}", cmd);
    let args: Vec<&str> = arg.split(" ").collect();
    let result = std::process::Command::new("ffmpeg").args(args).output().unwrap();
    match result.status.success() {
        true => Ok(()),
        false => {
                let err = name + ".mkv couldn't not be written.";
                Err(err)
            }
    }
}

fn deserialize(files: Vec<String>) -> Vec<AssFile> {
    let mut deserialized_files: Vec<AssFile> = vec![];
    for x in files {
        let file = fs::read_to_string(&x).unwrap();
        let ass: Result<AssFile, error::Error> = AssFile::get_fonts(file);
        if ass.is_ok() {
            deserialized_files.append(&mut vec![ass.unwrap()]);
        } else {
            println!("Failed to serialize: \"{}\".", x);
        }
    };
    deserialized_files
}

fn to_file_list(input: Vec<String>) -> Vec<String> {
    let mut file_list: Vec<String> = vec![];
    for x in input {
        if ! Path::new(&x).exists() {
            panic!("\"{}\" does not exist!", x);
        }
        for f in WalkDir::new(&x)
            .into_iter()
            .filter_map(|f| f.ok()) {
            if f.metadata().unwrap().is_file() {
                file_list.append(&mut vec![f.path().to_string_lossy().to_string()])
            };
        }
    };
    file_list
}
