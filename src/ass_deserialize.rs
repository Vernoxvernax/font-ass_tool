use std::vec;
use crate::error::Error;


pub struct AssFile {
    pub fonts: Vec<String>
}

impl AssFile {
    pub fn get_fonts(f: String) -> Result<AssFile, Error> {
        let styles = Self::get_styles(&f)?;
        let events = Self::get_event_lines(&f)?;
        
        let font_trim = Self::trim_to_fonts(styles, events);
        let fonts = font_trim.expect("Failed to trim to font_names");
        Ok(AssFile {fonts: fonts})
    }

    fn get_styles(f: &String) -> Result<Vec<String>, Error> {
        let mut header: Option<String> = None;
        let mut lines: Vec<String> = vec![];
        for line in f.lines() {
            if line.starts_with("[") && line.ends_with("]") && line.contains("Styles") {
                header = Some(line.to_string());
                continue
            }
            if line.starts_with("[") && line.ends_with("]") && line.contains("Events") {
                return Ok(lines)
            }
            if header.is_some() {
                if ! line.is_empty() && ! line.starts_with("Format:") {
                    lines.append(&mut vec![line.to_string()]);
                }
            }
        }
        Err(Error::MissingStylesInfo)
    }

    fn get_event_lines(f: &String) -> Result<Vec<String>, Error> {
        let mut header: Option<String> = None;
        let mut lines: Vec<String> = vec![];
        for line in f.lines() {
            if line.starts_with("[") && line.ends_with("]") && line.contains("Events") {
                header = Some(line.to_string());
                continue
            }
            if header.is_some() {
                if ! line.is_empty() && ! line.starts_with("Format:") {
                    lines.append(&mut vec![line.to_string()]);
                }
            }
        }
        if ! lines.is_empty() {
            Ok(lines)
        } else {
            Err(Error::MissingEvents)
        }
    }

    fn trim_to_fonts(styles: Vec<String>, events: Vec<String>) -> Result<Vec<String>, Error> {
        let mut fonts: Vec<String> = vec![];
        
        for line in styles {
            let mut font = String::new();
            let mut comma_ed: bool = false;
            for ch in line.trim_start_matches("Style: ").chars() {
                if ch == ',' {
                    if comma_ed {
                        break
                    } else {
                        comma_ed = true
                    }
                    continue
                }
                if comma_ed {
                    font.push_str(ch.to_string().as_str());
                }
            }
            if ! fonts.contains(&font.clone()) {
                fonts.append(&mut vec![font.clone()]);
            }
        }

        for line in events {
            let mut font = String::new();
            let mut styled: bool = false;
            let mut read_tag: bool = false;
            let mut record: bool = false;
            for ch in line.chars() {
                if read_tag && ch == 'f' {
                    continue
                } if read_tag && ch == 'n' {
                    record = true;
                    read_tag = false;
                    continue;
                } else {
                    read_tag = false;
                }
                if ch == '{' || ch == '}' {
                    if styled {
                        if ! font.is_empty() {
                            if ! fonts.contains(&font.clone()) {
                                fonts.append(&mut vec![font.clone()]);
                            }
                            font.clear()
                        }
                        styled = false
                    } else {
                        styled = true
                    }
                    continue
                }
                if ch == '\\' {
                    if ! font.is_empty() {
                        record = false;
                    }
                    read_tag = true;
                }
                if record {
                    font.push_str(ch.to_string().as_str());
                }
            }
        }
        if ! fonts.is_empty() {
            Ok(fonts)
        } else {
            Err(Error::FailedParsingFonts)
        }
    }
}