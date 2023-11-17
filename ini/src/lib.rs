#![forbid(unsafe_code)]

use std::collections::HashMap;

////////////////////////////////////////////////////////////////////////////////

pub type IniFile = HashMap<String, HashMap<String, String>>;

pub fn parse(content: &str) -> IniFile {
    if content.chars().all(|c| c.is_whitespace()) {
        return HashMap::new();
    }

    let mut ini: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut inner = HashMap::new();

    let mut current_ini_section = "";
    content
        .lines()
        .filter_map(|l| {
            let tl = l.trim();
            match tl {
                "" => None,
                _ => Some(tl),
            }
        })
        .for_each(|line| {
            if line.starts_with('[') && line.ends_with(']') {
                current_ini_section = line.trim_matches(|c| c == '[' || c == ']');
                if line.len() - current_ini_section.len() > 2 {
                    panic!("square brackets are not avaliable in section header");
                }
            } else if line.contains('=') {
                let (key, value) = line.split_at(line.find('=').expect("checked by cond"));
                let value = value.trim_start_matches(|c| c == '=').trim();
                if value.contains('=') {
                    panic!("= is not avaliable in value");
                }
                inner.insert(key.trim().to_string(), value.to_string());
            } else {
                inner.insert(line.to_string(), "".to_string());
            }

            if !current_ini_section.is_empty() {
                ini.entry(current_ini_section.to_string())
                    .or_default()
                    .extend(inner.clone());
                inner.clear();
            }
        });

    if ini.is_empty() {
        panic!("invalid .ini");
    }

    ini
}
