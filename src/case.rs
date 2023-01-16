use regex::Regex;

/// Uppercase first letter of each word, lowercase rest
pub fn upper_first(s: &str) -> String {
    s.split_whitespace()
        .map(|x| upper_first_once(x))
        .collect::<Vec<String>>()
        .join(" ")
}

/// Uppercase first letter of string, lowercase rest
pub fn upper_first_once(s: &str) -> String {
    let mut chars = s.chars();

    let first = match chars.next() {
        Some(x) => x.to_string(),
        None => return String::new(),
    };

    first.to_uppercase() + &chars.as_str().to_lowercase()
}

/// Kebab case, lowercase, only [a-z0-9-]
pub fn kebab_ascii(s: &str) -> String {
    //TODO Use lazy_static for regex
    s.to_lowercase()
        .chars()
        .filter_map(|ch| {
            if ch == ' ' {
                return Some("-".to_string());
            }

            let str_ch = ch.to_string();
            if Regex::new("[a-z0-9-]").unwrap().is_match(&str_ch) {
                Some(str_ch)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn upper_first_works() {
        assert_eq!(upper_first_once("abc dEF Ghi"), "Abc def ghi");

        assert_eq!(upper_first("abc dEF Ghi"), "Abc Def Ghi");
    }

    #[test]
    fn kebab_ascii_works() {
        assert_eq!(kebab_ascii("abc DE-F 02 $*)g"), "abc-de-f-02-g")
    }
}
