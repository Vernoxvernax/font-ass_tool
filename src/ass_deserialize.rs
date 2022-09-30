use std::vec;
use crate::error::Error;

#[derive(PartialEq, Clone, Debug)]
pub struct AssFont {
    pub facename: String,
    pub bold: bool,
    pub italic: bool,
    pub path: String
}

#[derive(Debug)]
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
                if ! line.is_empty() && ! line.starts_with("Format:") && ! line.starts_with("Comment:") {
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

        fn get_tags(line: String) -> Option<Vec<String>> {
            let mut styles: Vec<String> = vec![];
            let mut record: bool = false;
            let mut temp: String = String::new();
            for character in line.chars() {
                if character == '{' {
                    record = true;
                    continue;
                } else if character == '}' {
                    record = false;
                    styles.append(&mut vec![temp.clone()]);
                    temp.clear();
                    continue;
                }
                if record {
                    temp.push_str(character.to_string().as_str())
                }
            };
            let mut tags: Vec<String> = vec![];
            for style in styles {
                let splits = style.split_terminator("\\");
                for str in splits {
                    tags.append(&mut vec![("\\".to_owned() + str).to_string()]);
                }
            };
            Some(tags)
        }
        
        for line in events {
            if line.contains(r#"\fn"#) {
                let tags = get_tags(line);
                if tags.is_some() {
                    let mut bold: bool = false;
                    let mut italic: bool = false;
                    for tag in tags.unwrap() {
                        if tag == "\\b1" {
                            bold = true;
                        } else if tag == "\\b0" {
                            bold = false;
                        } else if tag == "\\i1" {
                            italic = true;
                        } else if tag == "\\i0" {
                            italic = false;
                        } else if tag.starts_with("\\fn") {
                            let font_str = tag.trim_start_matches("\\fn");
                            fonts.append(&mut vec![AssFont {
                                facename: font_str.to_string(),
                                bold,
                                italic,
                                path: "".to_string()
                            }]);
                            bold = false;
                            italic = true;
                        }
                    }
                } else {
                    continue
                }
            } else {
                continue
            }
        };
        if ! fonts.is_empty() {
            Ok(fonts)
        } else {
            Err(Error::FailedParsingFonts)
        }
    }
}