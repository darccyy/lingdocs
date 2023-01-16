use html_escape::encode_text as escape_html;
use regex::Regex;

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
                r#"<h{d} class="header"> {} </h{d}>"#,
                escape_html(rest),
                d = c.len(),
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
                    Some(format!("<p class=\"line\"> {} </p>\n", escape_html(s)))
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

fn format_statements(body: &str) -> String {
    use Statement::*;

    enum Statement {
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
    let mut curr_statement: Option<Statement> = None;
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
                '\\' => (),

                '{' if statement.is_none() => {
                    statement = Some(String::new());
                }

                '}' if statement.is_some() => {
                    // Unwrap should not fail
                    let stat = statement.unwrap();
                    let stat = stat.trim();

                    if let Some(curr_statement) = &curr_statement {
                        output.push_str(&match curr_statement {
                            Text(lang) => format!(
                                "<span class=\"language\">\
                                    <span class=\"name\"> {lang} </span>\
                                    <span class=\"text\"> {} </span>\
                                </span>",
                                stat
                            ),

                            Link(link) => {
                                format!(r#"<a class="link" href="{link}"> {} </a>"#, stat)
                            }

                            BroadIPA => format!(r#"<span class="ipa broad"> /{}/ </span>"#, stat),
                            NarrowIPA => format!(r#"<span class="ipa narrow"> [{}] </span>"#, stat),
                            Phoner => format!(r#"<code class="phoner"> {} </code>"#, stat),
                            // Double $ to not confuse regex later
                            Table => format!("(&lt;table&gt;){}(&lt;/table&gt;)", stat),
                            Replace => format!("{{$${}}}", stat),

                            Comment => String::new(),
                            Unknown => format!("{}", stat),
                        });
                    } else {
                        output.push_str(&stat);
                    }
                    println!("{}", output);

                    statement = None;
                    curr_statement = None;
                }

                _ => {
                    if let Some(stat) = &mut statement {
                        if let None = curr_statement {
                            curr_statement = Some(match ch {
                                '\'' => {
                                    curr_statement_building = true;
                                    Text(String::new())
                                }
                                '@' => {
                                    curr_statement_building = true;
                                    Link(String::new())
                                }
                                '/' => BroadIPA,
                                '[' => NarrowIPA,
                                '*' => Phoner,
                                '|' => Table,
                                '$' => Replace,
                                '#' => Comment,
                                _ => Unknown,
                            })
                        } else {
                            //TODO Tidy this cringe code
                            if curr_statement_building {
                                match &mut curr_statement {
                                    Some(Text(string) | Link(string)) if ch != ' ' => {
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
