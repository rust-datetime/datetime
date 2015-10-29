use std::ffi::OsStr;
use std::path::Path;


pub fn current_timezone() -> Option<String> {
    use std::fs::read_link;

    let link = match read_link("/etc/localtime") {
        Ok(link) => link,
        Err(_) => return None,
    };

    if let Some(tz) = extract_timezone(&*link) {
        if !tz.is_empty() {
            return Some(tz);
        }
    }

    None
}

fn extract_timezone(path: &Path) -> Option<String> {
    let mut bits = Vec::new();

    for pathlet in path.iter().rev().take_while(|c| is_tz_component(c)) {
        match pathlet.to_str() {
            Some(s) => bits.insert(0, s),
            None => return None,
        }
    }

    Some(bits.join("/"))
}

fn is_tz_component(component: &OsStr) -> bool {

    if let Some(component_str) = component.to_str() {
        let first_char = component_str.chars().next().unwrap();
        first_char.is_uppercase()
    }
    else {
        false
    }
}


#[cfg(test)]
mod test {
    use super::extract_timezone;
    use std::path::Path;

    #[test]
    fn two() {
        let timezone = extract_timezone(Path::new("/usr/share/zoneinfo/Europe/London"));
        assert_eq!(timezone, Some("Europe/London".to_string()));
    }

    #[test]
    fn one() {
        let timezone = extract_timezone(Path::new("/usr/share/zoneinfo/CST6CDT"));
        assert_eq!(timezone, Some("CST6CDT".to_string()));
    }
}
