//! Methods for determining the type of content that a tag is
//!
//! https://html.spec.whatwg.org/#kinds-of-content

/// Whether or not this tag is interactive content
///
/// https://html.spec.whatwg.org/#interactive-content-2
///
/// ```
/// assert_eq!(is_interactive_content(""))
/// ```
pub fn is_interactive_content (tag: &ElementTag) -> bool {
    unimplemented!()
}