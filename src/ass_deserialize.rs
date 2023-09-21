use std::vec;
use crate::error::Error;

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct AssFont {
  pub facename: String,
  pub bold: bool,
  pub italic: bool,
  pub path: String
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct AssStyle {
  pub name: String,
  pub font: AssFont
}

#[derive(Debug, Clone)]
pub struct AssFile {
  pub fonts: Vec<AssFont>
}

impl AssFile {
  pub fn get_fonts(f: String) -> Result<AssFile, Error> {
    let styles = Self::get_styles(&f)?;
    let events = Self::get_event_lines(&f)?;
    
    let font_trim = Self::trim_to_fonts(styles, events);
    let fonts = font_trim.expect("Failed to trim to font_names");
    Ok(AssFile {fonts})
  }

  fn get_styles(f: &str) -> Result<Vec<String>, Error> {
    let mut header: Option<String> = None;
    let mut lines: Vec<String> = vec![];
    for line in f.lines() {
      if line.starts_with('[') && line.ends_with(']') && line.contains("Styles") {
        header = Some(line.to_string());
        continue
      }
      if line.starts_with('[') && line.ends_with(']') && line.contains("Events") {
        return Ok(lines)
      }
      if header.is_some() && ! line.is_empty() && ! line.starts_with("Format:") {
        lines.append(&mut vec![line.to_string()]);
      }
    }
    Err(Error::MissingStylesInfo)
  }

  fn get_event_lines(f: &str) -> Result<Vec<String>, Error> {
    let mut header: Option<String> = None;
    let mut lines: Vec<String> = vec![];
    for line in f.lines() {
      if line.starts_with('[') && line.ends_with(']') {
        header = Some(line.to_string());
      } else if header == Some("[Events]".to_string()) && ! line.is_empty() && ! line.starts_with("Format:") && ! line.starts_with("Comment:") {
        lines.append(&mut vec![line.to_string()]);
        continue;
      }
    }
    if ! lines.is_empty() {
      Ok(lines)
    } else {
      Err(Error::MissingEvents)
    }
  }

  fn trim_to_fonts(style_lines: Vec<String>, events: Vec<String>) -> Result<Vec<AssFont>, Error> {
    let mut fonts: Vec<AssFont> = vec![];
    let mut styles: Vec<AssStyle> = vec![];
    
    for line in style_lines {
      let mut font = String::new();
      let mut bold: bool = false;
      let mut italic: bool = false;
      let mut comma_ed: u8 = 0;
      let mut style_name: String = String::new();

      for ch in line.trim_start_matches("Style: ").chars() {
        if ch == ',' {
          comma_ed += 1;
        } else if comma_ed == 0 {
          style_name.push(ch);
        } else if comma_ed == 1 {
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

      styles.append(&mut vec![AssStyle {
        name: style_name,
        font: assfont
      }]);
    }
    
    for line in events {
      let mut style_name: String = String::new();
      let mut facename: String = String::new();
      let mut comma_ed: u8 = 0;
      let mut bold: bool = false;
      let mut italic: bool = false;
      for ch in line.chars() {
        if ch == ',' {
          comma_ed += 1;
          
          if comma_ed == 4 {
            let style = if let Some(style) = styles.iter().find(| s | s.name == style_name) {
              style
            } else {
              println!("fdasfs");
              styles.get(0).unwrap()
            };
            facename = style.font.facename.clone();
            bold = style.font.bold;
            italic = style.font.italic;
            break;
          }
        } else if comma_ed == 3 {
          style_name.push(ch);
        }
      }

      if facename == "" {
        println!("{}", line);
      }

      if line.contains(r#"\fn"#) || line.contains(r#"\i"#) || line.contains(r#"\b"#) {
        let tags = get_tags(line.clone());
        if let Some(tagged) = tags {
          for tag in tagged {
            if tag == "\\b1" {
              bold = true;
            } else if tag == "\\b0" {
              bold = false;
            } else if tag == r#"\i1"# {
              italic = true;
            } else if tag == r#"\i0"# {
              italic = false;
            } else if tag.starts_with("\\fn") {
              facename = tag.trim_start_matches("\\fn").to_string();
            }
          }

          let assfont = AssFont {
            facename: facename.to_string(),
            bold,
            italic,
            path: "".to_string()
          };

          if ! fonts.contains(&assfont) {
            fonts.append(&mut vec![assfont]);
          }
        } else {
          continue
        }
      } else {
        let assfont = AssFont {
          facename: facename.to_string(),
          bold,
          italic,
          path: "".to_string()
        };

        if ! fonts.contains(&assfont) {
          fonts.append(&mut vec![assfont]);
        }
      }
    };

    if ! fonts.is_empty() {
      Ok(fonts)
    } else {
      Err(Error::FailedParsingFonts)
    }
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

  if styles.is_empty() {
    return None;
  }

  let mut tags: Vec<String> = vec![];
  for style in styles {
    let splits = style.split_terminator('\\');
    for str in splits {
      tags.append(&mut vec![("\\".to_owned() + str).to_string()]);
    }
  };
  
  Some(tags)
}
