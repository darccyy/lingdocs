use html_escape::encode_text as escape_html;
use regex::Regex;

use crate::{case, utils::separate_filename_ext};

pub fn ling_to_html(file: &str) -> String {
    use ListType::*;

    /// Kind of HTML list
    enum ListType {
        NoList,
        Unordered,
        Ordered,
    }

    // Standardize linebreaks
    let file = file.replace("\r\n", "\n");

    // Build values
    let mut body = Vec::<String>::new();
    let mut curr_list = NoList;

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
        if token != "." && token != "-" {
            match curr_list {
                NoList => (),
                Unordered => body.push("</ol>".to_string()),
                Ordered => body.push("</ul>".to_string()),
            }
            curr_list = NoList;
        }

        // Add tags if token matches
        //TODO Use lazy_static for instances of Regex::new - Add error message
        //TODO Add classes
        let maybe_push = match token {
            // Header
            c if Regex::new(r"^#+$").unwrap().is_match(&c) => Some(format!(
                r#"<h{d} class="header" id="{id}"> {} </h{d}>"#,
                escape_html(rest),
                id = case::kebab_ascii(rest.trim()),
                d = c.len() + 1,
            )),

            // Quote or note
            ">" => Some(format!(
                r#"<blockquote class="quote"> {} </blockquote>"#,
                escape_html(rest)
            )),

            // Hr
            "---" => Some("<hr />".to_string()),

            // Unordered list
            "-" => {
                // Add opening list tag if not active, and close other previous list if active
                let parent = match curr_list {
                    NoList => "<ul class=\"list\">\n",
                    Ordered => "",
                    Unordered => "</ol>\n<ul class=\"list\">\n",
                };
                curr_list = Ordered;

                Some(format!("{parent}<li> {} </li>", escape_html(rest)))
            }

            // Ordered list
            "." => {
                // Add opening list tag if not active, and close other previous list if active
                let parent = match curr_list {
                    NoList => "<ol class=\"list\">\n",
                    Ordered => "</ul>\n<ol class=\"list\">\n",
                    Unordered => "",
                };
                curr_list = Unordered;

                Some(format!(
                    r#"{parent}<li class="item"> {} </li>"#,
                    escape_html(rest)
                ))
            }

            // Normal line
            _ => {
                let s = line.trim();
                if !s.is_empty() {
                    //TODO Fix this
                    if s.starts_with("{|") && s.ends_with('}') {
                        // For tables
                        // Don't wrap in p tags
                        Some(escape_html(s).to_string())
                    } else {
                        // Wrap in p tags
                        Some(format!("<p class=\"line\"> {} </p>\n", escape_html(s)))
                    }
                } else {
                    None
                }
            }
        };

        if let Some(do_push) = maybe_push {
            body.push(do_push.trim().to_string());
        }
    }

    let body = format_statements(&body.join("\n"));

    let body = format_primatives(&body);

    body
}

#[derive(Debug)]
enum Format {
    Text(String),
    Link(String),
    BroadIPA,
    NarrowIPA,
    Phoner,
    Table,
    Replace,
    Comment,
    Unknown,
}

impl Format {
    pub fn from(ch: char) -> Self {
        use Format::*;

        match ch {
            '\'' => Text(String::new()),
            '@' => Link(String::new()),
            '/' => BroadIPA,
            '[' => NarrowIPA,
            '*' => Phoner,
            '|' => Table,
            '$' => Replace,
            '#' => Comment,
            _ => Unknown,
        }
    }

    pub fn format(&self, string: &str) -> String {
        use Format::*;

        match self {
            Text(lang) if lang.is_empty() => format!(
                "<span class=\"language no-name\">\
                    <span class=\"text\"> {} </span>\
                </span>",
                string
            ),

            Text(lang) => format!(
                "<span class=\"language with-name\">\
                    <span class=\"name\"> {} </span>\
                    <span class=\"text\"> {} </span>\
                </span>",
                lang.trim(),
                string,
            ),

            Link(link) => {
                format!(
                    r#"<a class="link" href="{}"> {} </a>"#,
                    format_link(link),
                    string
                )
            }

            BroadIPA => format!(
                "<span class=\"ipa broad\">\
                    <span class=\"delim before\"> / </span>\
                    <span class=\"text\"> {} </span>\
                    <span class=\"delim after\"> / </span>\
                </span>",
                string
            ),
            NarrowIPA => format!(
                "<span class=\"ipa narrow\">\
                    <span class=\"delim before\"> [ </span>\
                    <span class=\"text\"> {} </span>\
                    <span class=\"delim after\"> ] </span>\
                </span>",
                string
            ),
            Phoner => {
                let string = string.trim();

                format!(
                    r#"<code class="phoner"> {} </code>"#,
                    // Remove semicolon if is last character
                    if string.ends_with(';') {
                        remove_last_char(string).trim()
                    } else {
                        string
                    }
                )
            }
            // Double $ to not confuse regex later
            Table => format_table(string),
            Replace => format!("{{$${}}}", string),

            Comment => String::new(),
            Unknown => format!("{}", string),
        }
    }
}

/// Format link
///
/// Replaces `.ling` with `.html` file extension
fn format_link(link: &str) -> String {
    let (filename, mut ext) = separate_filename_ext(link);

    if ext.is_empty() {
        return filename;
    }

    if ext == "ling" {
        ext = "html";
    }

    filename + "." + ext
}

fn format_statements(body: &str) -> String {
    let mut curr_statement: Option<Format> = None;
    let mut curr_statement_building = false;

    // Build variables
    let mut output = String::new();
    let mut statement: Option<String> = None;
    let mut is_escaped = false;

    for ch in body.chars() {
        if is_escaped {
            output.push(ch);
        } else {
            match ch {
                '{' if statement.is_none() => {
                    statement = Some(String::new());
                }

                '}' if statement.is_some() => {
                    // Unwrap should not fail
                    let stat = statement.unwrap();
                    let stat = stat.trim();

                    if let Some(curr_statement) = &curr_statement {
                        output.push_str(&curr_statement.format(stat));
                    } else {
                        output.push_str(&stat);
                    }

                    statement = None;
                    curr_statement = None;
                }

                _ => {
                    if let Some(stat) = &mut statement {
                        if let None = curr_statement {
                            curr_statement_building = true;
                            curr_statement = Some(Format::from(ch));
                        } else {
                            //TODO Tidy this cringe code
                            if curr_statement_building {
                                match &mut curr_statement {
                                    Some(Format::Text(string) | Format::Link(string))
                                        if ch != ' ' =>
                                    {
                                        string.push(ch);
                                    }
                                    _ => {
                                        curr_statement_building = false;
                                    }
                                }
                            } else {
                                stat.push(ch);
                            }
                        }
                    } else {
                        output.push(ch);
                    }
                }
            }
        }

        // Escape next character
        if ch == '\\' && !is_escaped {
            is_escaped = true;
        } else {
            is_escaped = false;
        }
    }

    output
}

/// Format table from string
fn format_table(text: &str) -> String {
    // Build variables
    let mut table = Vec::<String>::new();
    let mut formats = Vec::<Option<Format>>::new();

    for (line_num, line) in text.lines().enumerate() {
        // Build row
        let mut row = Vec::<String>::new();

        for (col_num, cell) in line.split("|").enumerate() {
            if line_num == 0 {
                // Head
                // Get format for body cells in same column, from header
                //TODO Tidy this
                let mut chars = cell.chars();
                let mut push_format = None;
                if !cell.starts_with(' ') {
                    if let Some(ch) = chars.next() {
                        push_format = Some(Format::from(ch));
                    }
                }
                formats.push(push_format);

                // Add head cell to row
                row.push(format!(
                    "    <th class=\"cell head\"> {} </th>",
                    chars.as_str().trim()
                ));
            } else {
                // Body
                // Format from header
                let text = match formats.get(col_num) {
                    Some(Some(format)) => format.format(cell),
                    _ => cell.to_string(),
                };

                // Add body cell to row
                row.push(format!(
                    "    <td class=\"cell body\"> {} </td>",
                    text.trim()
                ));
            };
        }

        // Add row to table
        table.push(format!(
            "  <tr class=\"row\">\n\
                {}\n  </tr>\
        ",
            row.join("\n")
        ));
    }

    // Return table
    format!(
        "\
        \n<table class=\"table\">\n\
            {}\n\
        </table>\n\
    ",
        table.join("\n")
    )
}

fn format_primatives(body: &str) -> String {
    #[derive(Default)]
    /// Types of primative formats
    struct Primatives {
        italic: bool,
        bold: bool,
        underline: bool,
        strike: bool,
    }
    let mut prims = Primatives::default();

    // Build variables
    let mut output = String::new();
    let mut is_escaped = false;

    for ch in body.chars() {
        if is_escaped {
            output.push(ch);
        } else {
            match ch {
                // Non-escaped slash
                '\\' => (),

                // Italic
                '*' => {
                    output.push_str(if prims.italic {
                        "</i>"
                    } else {
                        r#"<i class="italics">"#
                    });
                    prims.italic = !prims.italic;
                }

                // Bold
                '^' => {
                    output.push_str(if prims.bold {
                        "</b>"
                    } else {
                        r#"<b class="bold">"#
                    });
                    prims.bold = !prims.bold;
                }

                // Underline
                '_' => {
                    output.push_str(if prims.underline {
                        "</u>"
                    } else {
                        r#"<u class="underline">"#
                    });
                    prims.underline = !prims.underline;
                }

                // Strike
                '~' => {
                    output.push_str(if prims.strike {
                        "</strike>"
                    } else {
                        r#"<strike class="strike">"#
                    });
                    prims.strike = !prims.strike;
                }

                // Other
                _ => output.push(ch),
            }
        }

        // Escape next character
        if ch == '\\' && !is_escaped {
            is_escaped = true;
        } else {
            is_escaped = false;
        }
    }

    output
}

/// Split string at linebreaks that are not inside 'statements' of curly braces
fn split_lines_preserve_statements(string: &str) -> Vec<String> {
    // Build variables
    let mut vec = Vec::new();
    let mut build = String::new();
    let mut is_statement = false;

    for ch in string.chars() {
        match ch {
            // Linebreak - new item if NOT in statement
            '\n' if !is_statement => {
                // Add item and reset build
                if !build.is_empty() {
                    vec.push(build);
                }
                build = String::new();
                continue;
            }

            // Toggle in_statement status
            '{' if !is_statement => is_statement = true,
            '}' if is_statement => is_statement = false,

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

fn remove_last_char(s: &str) -> &str {
    let mut chars = s.chars();
    chars.next_back();
    chars.as_str()
}

#[cfg(test)]
mod tests {
    use super::{remove_last_char as rlc, split_lines_preserve_statements as slps};

    #[test]
    fn remove_last_char_works() {
        assert_eq!(rlc("abc"), "ab");
        assert_eq!(rlc("a"), "");
        assert_eq!(rlc(""), "");
    }

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
