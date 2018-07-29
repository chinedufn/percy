#![feature(use_extern_macros)]
#![feature(proc_macro_non_items)]

extern crate inline_stylesheet_macro;

use inline_stylesheet_macro::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn css_classes_increment() {
        let class = css! {r#"
        :host {
            background-color: red;
        }
        "#};

        let class2 = css !{r#"
        :host {
            color: red;
        }
        :host > div { color: blue; }
        "#};

        assert_eq!(class, "_iss_0");
        assert_eq!(class2, "_iss_1");
    }
}
