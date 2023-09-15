use std::{fs, vec};
use std::path::Path;
use std::ffi::{CStr, c_void};
use std::os::raw::c_char;
use std::ptr::null_mut;
use std::process::{exit, ExitCode};
use ass_deserialize::AssFont;
use clap::{Arg, Command, ArgAction};
use walkdir::WalkDir;

use fontconfig_sys::{
  FcMatchPattern, FcResultMatch, FcSetSystem, FcResult, FcChar8, FcBool, FcConfig, FcPattern, FcFontSet, ffi_dispatch
};

#[cfg(not(feature = "dlopen"))]
use fontconfig_sys::{
  FcConfigSubstitute, FcDefaultSubstitute, FcPatternAddBool, FcFontSetAdd, FcPatternDuplicate, FcPatternGetString, FcFontSetSort,  FcPatternDestroy, FcConfigDestroy,
  FcPatternCreate, FcPatternAddInteger, FcConfigBuildFonts, FcInitLoadConfig, FcFontSetCreate, FcConfigGetFonts, FcPatternGetBool, FcFontSetDestroy, FcWeightFromOpenType,
};

#[cfg(feature = "dlopen")]
use fontconfig_sys::statics::LIB;

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
      .arg(
        Arg::new("force")
        .short('f')
        .long("force")
        .help("Tell FFmpeg to overwrite already existent output-files.")
        .required(false)
        .action(ArgAction::SetTrue)
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
      let force = run_matches.get_flag("force");
      let args = run_matches.get_many::<String>("file");
      let files = args.unwrap().map(|s| s.to_string()).collect::<Vec<_>>();
      let raw_files = to_file_list(files);
      let ass_files = deserialize(raw_files.clone());
      unsafe {
        // let config: *mut FcConfig = FcInitLoadConfig();
        let config = ffi_dispatch!(LIB, FcInitLoadConfig,);
        ffi_dispatch!(LIB, FcConfigBuildFonts, config);
        // FcConfigBuildFonts(config);
        for (file, name) in ass_files.iter().zip(raw_files.iter()) {
          if let Err(err) = remux_this(find_font_files(file.clone(), config), name.clone(), force) {
            println!("Error occurred for {}:\n  {}", name, err);
            return ExitCode::FAILURE;
          };
        }
        ffi_dispatch!(LIB, FcConfigDestroy, config);
        // FcConfigDestroy(config);
        return ExitCode::SUCCESS;
      }
    },
    Some(("check", check_matches)) => {
      let args = check_matches.get_many::<String>("file");
      let files = args.unwrap().map(|s| s.to_string()).collect::<Vec<_>>();
      let raw_files = to_file_list(files);
      let ass_files = deserialize(raw_files.clone());
      unsafe {
        let config = ffi_dispatch!(LIB, FcInitLoadConfig,);
        ffi_dispatch!(LIB, FcConfigBuildFonts, config);
        for (file, name) in ass_files.iter().zip(raw_files.iter()) {
          println!("{}:", name.clone());
          for font_file in find_font_files(file.clone(), config).fonts.iter() {
            println!("  {}      (b: {} i: {})       => {}", font_file.facename, font_file.bold, font_file.italic , font_file.path);
          };
          println!();
        }
        ffi_dispatch!(LIB, FcConfigDestroy, config);
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
    let clear_facename: &str = if font.facename.chars().nth(0).unwrap() == '@' {
      &font.facename[1..]
    } else {
      &font.facename
    };

    let family = clear_facename.to_lowercase();

    let weight: i32 = if font.bold {
      700
    } else {
      400
    };

    let slant: i32 = if font.italic {
      110
    } else {
      0
    };

    unsafe {
      let pattern = ffi_dispatch!(LIB, FcPatternCreate,);
      // let pattern = FcPatternCreate() as *mut FcPattern;
      if pattern.is_null() {
        continue;
      }

      ffi_dispatch!(LIB, FcPatternAddBool, pattern, FC_OUTLINE.as_ptr() as *mut c_char, true as FcBool);
      ffi_dispatch!(LIB, FcPatternAddInteger, pattern, FC_SLANT.as_ptr() as *mut c_char, slant);
      ffi_dispatch!(LIB, FcPatternAddInteger, pattern, FC_WEIGHT.as_ptr() as *mut c_char, ffi_dispatch!(LIB, FcWeightFromOpenType, weight));
      // FcPatternAddBool(pattern, FC_OUTLINE.as_ptr() as *mut c_char, true as FcBool);
      // FcPatternAddInteger(pattern, FC_SLANT.as_ptr() as *mut c_char, slant);
      // FcPatternAddInteger(pattern, FC_WEIGHT.as_ptr() as *mut c_char, FcWeightFromOpenType(weight));
      
      ffi_dispatch!(LIB, FcDefaultSubstitute, pattern);
      // FcDefaultSubstitute(pattern);
      if ffi_dispatch!(LIB, FcConfigSubstitute, config, pattern, FcMatchPattern) != 1 {
        continue;
      }
      // if FcConfigSubstitute(config, pattern, FcMatchPattern) != 1 {
      //   continue;
      // }

      let fset = ffi_dispatch!(LIB, FcFontSetCreate,);
      fcfind(ffi_dispatch!(LIB, FcConfigGetFonts, config, FcSetSystem), fset, &family);
      // let fset: *mut FcFontSet = FcFontSetCreate();
      // // fcfind(FcConfigGetFonts(config, FcSetApplication), fset, &family);
      // fcfind(FcConfigGetFonts(config, FcSetSystem), fset, &family);
      
      let result: *mut FcResult = &mut 0;
      let mut sets: *mut FcFontSet = { fset };

      let matches = ffi_dispatch!(LIB, FcFontSetSort, config, &mut sets, 1, pattern, false as FcBool, std::ptr::null_mut(), result);
      // let matches: *mut FcFontSet = FcFontSetSort(config, &mut sets, 1, pattern, false as FcBool, std::ptr::null_mut(), result);
      if (*matches).nfont == 0 {
        assfont.path = "Nothing found.".to_string();
        fonts.append(&mut vec![assfont]);
        continue;
      };

      ffi_dispatch!(LIB, FcFontSetDestroy, fset);
      ffi_dispatch!(LIB, FcPatternDestroy, pattern);
      // FcFontSetDestroy(fset);
      // FcPatternDestroy(pattern);

      let matching = *(*matches).fonts.offset(0);
      
      let mut file: *mut FcChar8 = &mut 0;
      if ffi_dispatch!(LIB, FcPatternGetString, matching, FC_FILE.as_ptr() as *mut c_char, 0, &mut file) != FcResultMatch {
        continue;
      }
      // if FcPatternGetString(matching, FC_FILE.as_ptr() as *mut c_char, 0, &mut file) != FcResultMatch {
      //   continue;
      // }

      assfont.path = std::str::from_utf8(CStr::from_ptr(file as *const c_char).to_bytes()).unwrap().to_owned();

      if cfg!(windows) {
        assfont.path = assfont.path.replace("/", "\\").replace("\\ ", " ");
      }

      assfont.path = if assfont.path.contains(' ') {
        "\"".to_owned() + &assfont.path + "\""
      } else {
        assfont.path
      };

      fonts.append(&mut vec![assfont]);
      ffi_dispatch!(LIB, FcFontSetDestroy, matches);
      // FcFontSetDestroy(matches);
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
    
      if ffi_dispatch!(LIB, FcPatternGetBool, pattern, FC_OUTLINE.as_ptr() as *mut c_char, 0, val) != FcResultMatch || *val != true as FcBool {
        continue;
      }
      // if FcPatternGetBool(pattern, FC_OUTLINE.as_ptr() as *mut c_char, 0, val) != FcResultMatch || *val != true as FcBool {
      //   continue;
      // };
      
      if pattern_match(pattern, FC_FULLNAME, family) || pattern_match(pattern, FC_FAMILY, family) {
        ffi_dispatch!(LIB, FcFontSetAdd, fset, ffi_dispatch!(LIB, FcPatternDuplicate, pattern));
        // FcFontSetAdd(fset, FcPatternDuplicate(pattern));
      };
    }
  }
}

fn pattern_match(pat: *mut c_void, field: &'static [u8], name: &String) -> bool {
  unsafe {
    let mut str: *mut FcChar8 = null_mut();
    for index in 0.. {
      if ffi_dispatch!(LIB, FcPatternGetString, pat, field.as_ptr() as *mut c_char, index, &mut str) == FcResultMatch {
      // if FcPatternGetString(pat, field.as_ptr() as *mut c_char, index, &mut str) == FcResultMatch {
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

fn remux_this(file: AssFile, name: String, force: bool) -> Result<(), String> {
  if ! force && Path::new(format!("{}.mkv", name).as_str()).exists() {
    println!("{}.mkv already exists.", name);
    return Ok(());
  }
  let mut duppl_check = String::new();
  let mut args: Vec<&str> = vec![];
  args.append(&mut vec!["-i"]);
  let input = name.clone();
  args.append(&mut vec![&input]);

  let mut cmd = String::new();
  let mut track_index = 1;
  for assfont in file.fonts {
    if duppl_check.contains(&assfont.path) {
      continue
    } else if assfont.path == "Nothing found." {
      println!("\"{}\" could not be found on your system!", assfont.facename);
      continue;
    }
    cmd = cmd.to_owned() + " -attach " + assfont.path.as_str();

    let path = assfont.path.replace("\"", "").to_lowercase();
    if path.ends_with(".ttf") {
      cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " mimetype=application/x-truetype-font";
    } else if path.ends_with(".otf") {
      cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " mimetype=application/x-font-opentype";
    } else if path.ends_with(".ttc") {
      cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " mimetype=application/x-truetype-collection";
    }

    let filename = if cfg!(windows) {
      path.split('\\').last().unwrap()
    } else {
      path.split('/').last().unwrap()
    };

    cmd = cmd.to_owned() + " -metadata:s:" + track_index.to_string().as_str() + " filename=\"" + filename + "\"";

    track_index += 1;
    duppl_check.push_str(&assfont.path);
  };

  let output = name + ".mkv";
  cmd = cmd.trim().to_string();

  if cmd.is_empty() {
    return Err("None of the required fonts could be found!".to_string());
  }

  let vector_string = controlled_space_splitting(cmd);
  let mut vector_str = vector_string.iter().map(|s| s.as_str()).collect();

  args.append(&mut vector_str);
  args.append(&mut vec![&output]);
  
  if force {
    args.append(&mut vec!["-y"])
  } else {
    args.append(&mut vec!["-n"]);
  }

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

fn controlled_space_splitting(input: String) -> Vec<String> {
  let mut output: Vec<String> = vec![];
  let mut temp: String = String::new();
  let mut quotations = false;
  for ch in input.chars() {
    if ch == '\"' {
      quotations = !quotations;
    } else if ch == ' ' && ! quotations {
      if !temp.is_empty() {
        output.push(temp.clone());
        temp.clear();
      }
    } else {
      temp.push(ch);
    }
  }

  if !temp.is_empty() {
    output.push(temp);
  }

  output
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
