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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn upper_first_works() {
        assert_eq!(upper_first_once("abc dEF Ghi"), "Abc def ghi");

        assert_eq!(upper_first("abc dEF Ghi"), "Abc Def Ghi");
    }
}
