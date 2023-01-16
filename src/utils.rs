/// Separate filename into name and extension, separately
///
/// All characters after last dot are included in extension
///
/// All characters (including dots), except everything after last dot, are included in filename
pub fn separate_filename_ext(filename: &str) -> (String, &str) {
    let mut split_at_dot = filename.split('.');

    // Last item
    let ext = split_at_dot.next_back().unwrap_or("");
    // Everything else (before last item)
    let rest = split_at_dot.collect::<Vec<_>>();

    if rest.is_empty() {
        // No extension
        // Use extension as filename, with extension being blank
        (ext.to_string(), "")
    } else {
        // Filename and extension
        (rest.join("."), ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separate_filename_ext_works() {
        assert_eq!(separate_filename_ext("abc.def"), ("abc".to_string(), "def"));

        assert_eq!(separate_filename_ext("abc"), ("abc".to_string(), ""));

        assert_eq!(
            separate_filename_ext("abc.def.ghi"),
            ("abc.def".to_string(), "ghi")
        );

        assert_eq!(
            separate_filename_ext("abc.def.ghi.jkl"),
            ("abc.def.ghi".to_string(), "jkl")
        );
    }
}
