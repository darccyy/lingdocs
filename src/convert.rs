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
    // ? Convert to strings ?
    // ? Convert to vec ?
    let lines = file.lines();
    for line in lines {
        // Split line into `token` and `rest` at first space
        // If no space, token is line, rest is empty
        // ? Invert this ^^ ?
        let (token, rest) = match line.find(' ') {
            Some(pos) => line.split_at(pos),
            None => (line, ""),
        };

        // Add closing list tag, if token does not match with list pattern
        // if token == "." || token == "-" {}

        // Add tags if token matches
        //TODO Use lazy_static for instances of Regex::new - Add error message
        let maybe_push = match token {
            // Header
            c if Regex::new(r"^#+$").unwrap().is_match(c) => Some(format!(
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
