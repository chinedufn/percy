#![feature(use_extern_macros)]
#![feature(proc_macro_non_items)]

extern crate inline_stylesheet_macro;

use inline_stylesheet_macro::css;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn css_classes_increment() {
        let class = css! {r#"
        :host {
            background-color: red;
        }
        "#};

        let class2 = css!{r#"
        :host {
            color: red;
        }
        :host > div { color: blue; }
        "#};

        assert_eq!(class, "_iss_0");
        assert_eq!(class2, "_iss_1");
    }

    #[test]
    fn writes_to_provided_file() {
        Command::new("cargo")
            .env("OUTPUT_CSS", "/tmp/percy-test-css.css")
            .arg("run")
            .args(&["-p", "test-stylesheet-fixture"])
            .spawn()
            .unwrap()
            .wait();

        let mut file = File::open("/tmp/percy-test-css.css").unwrap();
        let mut css = String::new();
        file.read_to_string(&mut css).unwrap();

        assert_eq!(
            css.replace(" ", "").replace("\n", ""),
            r#"
        ._iss_0 {
            color: red;
            background-color: blue;
        }
        "#.replace(" ", "")
                .replace("\n", "")
        );

        fs::remove_file("/tmp/percy-test-css.css");
    }
}
