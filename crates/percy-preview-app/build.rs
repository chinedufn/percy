use std::collections::HashSet;
use std::path::{PathBuf};
use sunbeam_build::{files_in_dir_recursive_ending_with, SunbeamConfig};

fn main() {
    let files = files_in_dir_recursive_ending_with(views_dir(), ".rs");

    let config = include_str!("Sunbeam.yml");
    let config: SunbeamConfig = serde_yaml::from_str(config).unwrap();

    let css = sunbeam_build::parse_rust_files(files, &config).unwrap();

    let mut all_classes = vec![];

    for class in css.iter() {
        all_classes.push(&class.class_name);
    }
    let all_classes: HashSet<&String> = all_classes.into_iter().collect();

    let css_count = all_classes.len();
    let mut all_class_strings = "".to_string();

    for class in all_classes {
        all_class_strings += "css!(\"";
        all_class_strings += class;
        all_class_strings += "\"),";
    }

    let all_classes = format!(
        r#"
use sunbeam::css;

/// All of the sunbeam CSS that this crate uses.
pub fn all() -> [&'static str; {css_count}] {{
    [{all_class_strings}]
}}
"#,
    );

    std::fs::write(out_dir().join("all_sunbeam_css.rs"), all_classes).unwrap();
}

fn views_dir() -> PathBuf {
    crate_dir().join("src/view")
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn out_dir() -> PathBuf {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    PathBuf::from(out_dir)
}
