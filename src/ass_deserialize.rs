use std::vec;
use crate::error::Error;

#[derive(PartialEq, Clone)]
pub struct AssFont {
    pub facename: String,
    pub bold: bool,
    pub italic: bool,
    pub path: String
}

pub struct AssFile {
    pub fonts: Vec<AssFont>
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

    fn trim_to_fonts(styles: Vec<String>, events: Vec<String>) -> Result<Vec<AssFont>, Error> {
        let mut fonts: Vec<AssFont> = vec![];
        
        for line in styles {
            let mut font = String::new();
            let mut bold: bool = false;
            let mut italic: bool = false;
            let mut comma_ed: u8 = 0;
            for ch in line.trim_start_matches("Style: ").chars() {
                if ch == ',' {
                    comma_ed += 1;
                    continue
                }
                if comma_ed == 1 {
                    font.push_str(ch.to_string().as_str());
                } else if comma_ed == 7 {
                    if ch == '0' {
                        bold = false;
                    } else {
                        bold = true;
                    }
                } else if comma_ed == 8 {
                    if ch == '0' {
                        italic = false;
                    } else {
                        italic = true;
                    }
                }
            }
            let assfont: AssFont = AssFont {
                facename: font,
                bold,
                italic,
                path: "".to_string()
            };
            if ! fonts.contains(&assfont) {
                fonts.append(&mut vec![assfont]);
            }
        }

        for line in events {
            let mut font = String::new();
            let mut bold: bool = false;
            let mut bold_check: bool = false;
            let mut italic: bool = false;
            let mut italic_check: bool = false;
            let mut styled: bool = false;
            let mut read_tag: bool = false;
            let mut record: bool = false;
            for ch in line.chars() {
                if italic_check || bold_check {
                    if italic_check {
                        if ch == '0' {
                            italic = false
                        } else if ch == '1' {
                            italic = true
                        }
                    } else {
                        if ch == '0' {
                            bold = false
                        } else if ch == '1' {
                            bold = true
                        }
                    }
                    (italic_check, bold_check) = (false, false);
                    continue
                } else if read_tag && ch == 'f' {
                    continue
                } else if read_tag && ch == 'n' {
                    record = true;
                    read_tag = false;
                    continue;
                } else if read_tag && ch == 'i' {
                    italic_check = true
                } else if read_tag && ch == 'b' {
                    bold_check = true
                } else {
                    read_tag = false;
                }
                if ch == '{' || ch == '}' {
                    if styled {
                        if ! font.is_empty() {
                            let assfont = AssFont {
                                facename: font.clone(),
                                bold,
                                italic,
                                path: "".to_string()
                            };
                            if ! fonts.contains(&assfont) {
                                fonts.append(&mut vec![assfont]);
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