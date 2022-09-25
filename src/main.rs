use std::fs;
use std::path::Path;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;
use std::ptr::{null_mut};
use std::str::from_utf8_unchecked;
use clap::{Arg, Command, ArgAction};
use walkdir::WalkDir;
use fontconfig::fontconfig::{
    FcMatchPattern, FcResultMatch, FcChar8,
    FcPatternFormat, FcNameParse, FcConfigSubstitute, FcDefaultSubstitute, FcFontMatch
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
                remux_this(find_font_files(file.clone()), name.clone()).expect("Failed to remux file.");
            }
        },
        Some(("check", check_matches)) => {
            let args = check_matches.get_many::<String>("file");
            let files = args.unwrap().map(|s| s.to_string()).collect::<Vec<_>>();
            let raw_files = to_file_list(files);
            let ass_files = deserialize(raw_files.clone());
            for (file, name) in ass_files.iter().zip(raw_files.iter()) {
                println!("{}:", name.clone());
                for (font_file, font_name) in find_font_files(file.clone()).fonts.iter().zip(file.fonts.iter()) {
                    println!("  {} => {}", font_name, font_file);
                };
                println!("");
            }
        }
        _ => unreachable!(),
    }
}

fn find_font_files(file: &AssFile) -> AssFile {
    let mut fonts: Vec<String> = vec![];
    for font in &file.fonts {
        let font_cstr = CString::new(font.clone()).unwrap();
        unsafe {
            let pattern = FcNameParse(font_cstr.as_ptr() as *mut FcChar8);
            FcConfigSubstitute(null_mut(), pattern, FcMatchPattern);
            FcDefaultSubstitute(pattern);
            let mut result = 0;
            let matched = FcFontMatch(null_mut(), pattern, &mut result);
            if result == FcResultMatch {
                let pattern2 = CString::new("%{file}").unwrap();
                let fonting = FcPatternFormat(matched, pattern2.as_ptr() as *const u8);
                let string = CStr::from_ptr(fonting as *const c_char).to_bytes_with_nul();
                fonts.append(&mut vec![from_utf8_unchecked(string).replace("\0", "").trim().to_string()]);
            } else {
                panic!("No match has been found for {}", font);
            }
        }
    };
    AssFile { fonts }
}

fn remux_this(file: AssFile, name: String) -> Result<(), String> {
    let mut duppl_check = String::new();
    let mut cmd = "-i ".to_owned() + name.as_str();
    let mut track_index = 1;
    for font_files in file.fonts {
        if duppl_check.contains(&font_files) {
            continue
        }
        cmd = cmd.to_owned() + " -attach " + font_files.as_str();
        if font_files.ends_with(".ttf\0") {
            cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " mimetype=application/x-truetype-font";
        } else if font_files.ends_with(".otf\0") {
            cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " mimetype=application/x-font-opentype";
        }
        track_index += 1;
        duppl_check.push_str(&font_files);
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
        // println!("{:?}", file);
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
