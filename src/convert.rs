use html_escape::encode_text as escape_html;
use regex::Regex;

// use ListType::*;

/// Kind of HTML list
#[allow(dead_code)]
enum ListType {
    NoList,
    Unordered,
    Ordered,
}

pub fn ling_to_html(file: &str) -> String {
    // Standardize linebreaks
    let file = file.replace("\r\n", "\n");

    // Build values
    let mut body = Vec::<String>::new();
    // let curr_list = NoList;

    // Loop lines in file
    let lines = split_lines_preserve_statements(&file);
    for line in lines {
        // Split line into `token` and `rest` at first space
        // If no space, token is line, rest is empty
        let (token, rest) = match line.find(' ') {
            Some(pos) => line.split_at(pos),
            None => (line.as_str(), ""),
        };

        // Add closing list tag, if token does not match with list pattern
        // if token == "." || token == "-" {}

        // Add tags if token matches
        //TODO Use lazy_static for instances of Regex::new - Add error message
        let maybe_push = match token {
            // Header
            c if Regex::new(r"^#+$").unwrap().is_match(&c) => Some(format!(
                "<h{d}> {} </h{d}>",
                escape_html(rest),
                d = c.len() + 1,
            )),

            // Normal line
            _ => {
                let s = line.trim();
                if !s.is_empty() {
                    Some(format!("<p> {} </p>\n", escape_html(s)))
                } else {
                    None
                }
            }
        };

        if let Some(do_push) = maybe_push {
            body.push(do_push.trim().to_string());
        }
    }

    body.join("\n")
}

/// Split string at linebreaks that are not inside 'statements' of curly braces
fn split_lines_preserve_statements(string: &str) -> Vec<String> {
    // Build variables
    let mut vec = Vec::new();
    let mut build = String::new();
    let mut in_statement = false;

    for ch in string.chars() {
        match ch {
            // Linebreak - new item if NOT in statement
            '\n' if !in_statement => {
                // Add item and reset build
                if !build.is_empty() {
                    vec.push(build);
                }
                build = String::new();
                continue;
            }

            // Toggle in_statement status
            '{' if !in_statement => in_statement = true,
            '}' if in_statement => in_statement = false,

            _ => (),
        }

        // Add character
        build.push(ch);
    }

    // Add final item
    if !build.is_empty() {
        vec.push(build);
    }

    vec
}

#[cfg(test)]
mod tests {
    use super::split_lines_preserve_statements as slps;

    #[test]
    fn split_lines_preserve_statements_works() {
        assert_eq!(slps("abc\ndef\nghi"), vec!["abc", "def", "ghi"]);

        assert_eq!(slps("{abc\ndef}\nghi"), vec!["{abc\ndef}", "ghi"]);

        assert_eq!(
            slps("abc\nde{f\nghi\n\n\njkl}\n123"),
            vec!["abc", "de{f\nghi\n\n\njkl}", "123"]
        );

        assert_eq!(slps("abc\nd{ef\nghi"), vec!["abc", "d{ef\nghi"]);

        assert_eq!(slps("abc\nd{}ef\nghi"), vec!["abc", "d{}ef", "ghi"]);

        assert_eq!(slps("abc\nd}ef\nghi"), vec!["abc", "d}ef", "ghi"]);
    }
}
