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
    use std::process::Command;
    use std::fs::OpenOptions;
    use std::io::Write;

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

    // TODO: Looks like this test will sometimes fail depending on if the test-css-rs-fixture crate
    // gets rebuild or not. So the first run might pass but then subsequent runs might not..
    #[test]
    fn writes_to_provided_file() {
        // We overwrite the file since at this time it looks like our procedural macro won't
        // run again if the file hasn't changed or been overwritten.
        // This fixes a problem where the first time you ran this test it would work but then after
        // that it would fail because the `css!` procedural macro wasn't getting run again since the file
        // hadn't changed changed.
        let fixture = "../test-css-rs-fixture/src/main.rs";
        let mut css_rs_fixture = File::open(fixture)
            .unwrap();
        let mut contents = String::new();
        css_rs_fixture.read_to_string(&mut contents).unwrap();
        fs::write(fixture, contents.as_bytes()).unwrap();

        // Run cargo build and verify that our CSS gets extracted

        Command::new("cargo")
            .env("OUTPUT_CSS", "/tmp/percy-test-css.css")
            .arg("build")
            .args(&["-p", "test-css-rs-fixture"])
            .spawn()
            .unwrap()
            .wait()
            .unwrap();

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

        fs::remove_file("/tmp/percy-test-css.css").unwrap();
    }
}
