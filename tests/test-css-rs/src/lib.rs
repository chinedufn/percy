#![feature(use_extern_macros)]
#![feature(proc_macro_non_items)]

extern crate css_rs_macro;

use css_rs_macro::css;

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

        assert_eq!(class, "_css_rs_0");
        assert_eq!(class2, "_css_rs_1");
    }

    #[test]
    fn writes_to_provided_file() {
        Command::new("cargo")
            .env("OUTPUT_CSS", "/tmp/percy-test-css.css")
            .arg("run")
            .args(&["-p", "test-css-rs-fixture"])
            .spawn()
            .unwrap()
            .wait();

        let mut file = File::open("/tmp/percy-test-css.css").unwrap();
        let mut css = String::new();
        file.read_to_string(&mut css).unwrap();

        assert_eq!(
            css.replace(" ", "").replace("\n", ""),
            r#"
        ._css_rs_0 {
            color: red;
            background-color: blue;
        }
        ._css_rs_1 {
            display: flex;
        }
        "#.replace(" ", "")
            .replace("\n", "")
        );

        fs::remove_file("/tmp/percy-test-css.css");
    }
}
