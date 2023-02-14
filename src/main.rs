use std::{fs, vec};
use std::path::Path;
use std::ffi::{CStr, c_void};
use std::os::raw::c_char;
use std::ptr::null_mut;
use std::process::{exit, ExitCode};
use ass_deserialize::AssFont;
use clap::{Arg, Command, ArgAction};
use walkdir::WalkDir;
use fontconfig::fontconfig::{
    FcMatchPattern, FcResultMatch, FcSetSystem, FcResult, FcChar8, FcBool, FcConfig, FcPattern, FcFontSet,
    FcConfigSubstitute, FcDefaultSubstitute, FcPatternAddBool, FcFontSetAdd, FcPatternDuplicate, FcPatternGetString, FcFontSetSort,  FcPatternDestroy, FcConfigDestroy,
    FcPatternCreate, FcPatternAddInteger, FcConfigBuildFonts, FcInitLoadConfig, FcFontSetCreate, FcConfigGetFonts, FcPatternGetBool, FcFontSetDestroy,
};

pub mod ass_deserialize;
pub mod error;
use crate::ass_deserialize::AssFile;
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> ExitCode {
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
            .about("Analyze a subtitle file written in ASS, and mux it with the required fonts into a matroska container.")
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
            unsafe {
                let config: *mut FcConfig = FcInitLoadConfig();
                FcConfigBuildFonts(config);
                for (file, name) in ass_files.iter().zip(raw_files.iter()) {
                    if let Err(err) = remux_this(find_font_files(file.clone(), config), name.clone()) {
                        println!("Error occurred for {}:\n  {}", name, err);
                        return ExitCode::FAILURE;
                    };
                }
                FcConfigDestroy(config);
                return ExitCode::SUCCESS;
            }
        },
        Some(("check", check_matches)) => {
            let args = check_matches.get_many::<String>("file");
            let files = args.unwrap().map(|s| s.to_string()).collect::<Vec<_>>();
            let raw_files = to_file_list(files);
            let ass_files = deserialize(raw_files.clone());
            unsafe {
                let config: *mut FcConfig = FcInitLoadConfig();
                FcConfigBuildFonts(config);
                for (file, name) in ass_files.iter().zip(raw_files.iter()) {
                    println!("{}:", name.clone());
                    for font_file in find_font_files(file.clone(), config).fonts.iter() {
                        println!("  {}      (b: {} i: {})       => {}", font_file.facename, font_file.bold, font_file.italic , font_file.path);
                    };
                    println!();
                }
                FcConfigDestroy(config);
                return ExitCode::SUCCESS;
            }
        }
        _ => unreachable!(),
    }
}

static FC_OUTLINE: &[u8] = b"outline\0";
static FC_FULLNAME: &[u8] = b"fullname\0";
static FC_FAMILY: &[u8] = b"family\0";
static FC_FILE: &[u8] = b"file\0";
static FC_WEIGHT: &[u8] = b"weight\0";
static FC_SLANT: &[u8] = b"slant\0";

fn find_font_files(file: AssFile, config: *mut FcConfig) -> AssFile {
    let mut fonts: Vec<AssFont> = vec![];
    for font in &file.fonts {
        let mut assfont = font.clone();
        let clear_facename: &str = if font.facename.chars().nth(1).unwrap() == '@' {
            &font.facename[1..]
        } else {
            &font.facename
        };

        let family = clear_facename.to_lowercase();

        let weight: i32 = if font.bold {
            200
        } else {
            80
        };

        let slant: i32 = if font.italic {
            110
        } else {
            0
        };

        unsafe {
            let pattern = FcPatternCreate() as *mut FcPattern;

            FcPatternAddBool(pattern, FC_OUTLINE.as_ptr() as *mut c_char, true as FcBool);
            FcPatternAddInteger(pattern, FC_SLANT.as_ptr() as *mut c_char, slant);
            FcPatternAddInteger(pattern, FC_WEIGHT.as_ptr() as *mut c_char, weight);
            FcDefaultSubstitute(pattern);

            if ! FcConfigSubstitute(config, pattern, FcMatchPattern) == 1 {
                continue;
            }

            let fset: *mut FcFontSet = FcFontSetCreate();
            fcfind(FcConfigGetFonts(config, FcSetSystem), fset, &family);
            
            let result: *mut FcResult = &mut 0;
            let mut sets: *mut FcFontSet = { fset };

            let matches: *mut FcFontSet = FcFontSetSort(config, &mut sets, 1, pattern, 0, std::ptr::null_mut(), result);
            if (*matches).nfont == 0 {
                assfont.path = "Nothing found.".to_string();
                fonts.append(&mut vec![assfont]);
                continue;
            };

            FcFontSetDestroy(fset);
            FcPatternDestroy(pattern);

            let matching = *(*matches).fonts.offset(0);
            
            let mut file: *mut FcChar8 = &mut 0;
            if FcPatternGetString(matching, FC_FILE.as_ptr() as *mut c_char, 0, &mut file) != FcResultMatch {
                continue;
            }

            assfont.path = std::str::from_utf8(CStr::from_ptr(file as *const c_char).to_bytes()).unwrap().to_owned();
            fonts.append(&mut vec![assfont]);
            FcFontSetDestroy(matches);
        }
    };
    AssFile { fonts }
}

fn fcfind(src: *mut FcFontSet, fset: *mut FcFontSet, family: &String) {
    unsafe {       
        for i in 0..((*src).nfont as isize) {
            let pattern: *mut FcPattern = *(*src).fonts.offset(i);
            let mut value = 0;
            let val: *mut FcBool = &mut value;
        
            if FcPatternGetBool(pattern, FC_OUTLINE.as_ptr() as *mut c_char, 0, val) != FcResultMatch || *val != 1 {
                continue;
            };
            
            if pattern_match(pattern, FC_FULLNAME, family) || pattern_match(pattern, FC_FAMILY, family) {
                FcFontSetAdd(fset, FcPatternDuplicate(pattern));
            };
        }
    }
}

fn pattern_match(pat: *mut c_void, field: &'static [u8], name: &String) -> bool {
    unsafe {
        for index in 0.. {
            let mut str: *mut FcChar8 = null_mut();
            if FcPatternGetString(pat, field.as_ptr() as *mut c_char, index, &mut str) == FcResultMatch {
                let sstr: &String = &std::str::from_utf8(CStr::from_ptr(str as *const c_char).to_bytes()).unwrap().to_owned().to_lowercase();
                if name == sstr {
                    return true;
                }
            } else {
                return false;
            }
        }
        false
    }
}

fn remux_this(file: AssFile, name: String) -> Result<(), String> {
    if Path::new(format!("{}.mkv", name).as_str()).exists() {
        println!("{}.mkv already exists.", name);
        return Ok(());
    }
    let mut duppl_check = String::new();
    let mut args: Vec<&str> = vec![];
    args.append(&mut vec!["-i"]);
    let input = name.clone();
    args.append(&mut vec![&input]);
    let mut cmd = "".to_string();
    let mut track_index = 1;
    for assfont in file.fonts {
        if duppl_check.contains(&assfont.path) {
            continue
        } else if assfont.path == "Nothing found." {
            return Err(format!("\"{}\" could not be found on your system.", assfont.facename));
        }
        cmd = cmd.to_owned() + " -attach " + assfont.path.as_str();
        if assfont.path.ends_with(".ttf") {
            cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " mimetype=application/x-truetype-font";
        } else if assfont.path.ends_with(".otf") {
            cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " mimetype=application/x-font-opentype";
        } else if assfont.path.ends_with(".ttc") {
            cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " mimetype=application/x-truetype-collection";
        }
        track_index += 1;
        duppl_check.push_str(&assfont.path);
    };
    let output = name + ".mkv";
    cmd = cmd.trim().to_string();
    args.append(&mut cmd.split(' ').collect());
    args.append(&mut vec![&output]);
    args.append(&mut vec!["-n"]);
    // println!("{:?}", args);
    let result = std::process::Command::new("ffmpeg").args(args).output().unwrap();
    match result.status.success() {
        true => Ok(()),
        false => {
                Err(String::from_utf8(result.stderr).unwrap())
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
            println!("\"{}\" does not exist!", x);
            exit(1);
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
