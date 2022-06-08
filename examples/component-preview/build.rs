fn main() {
    css::build_all_css();
}

mod css {
    use std::collections::HashSet;
    use std::path::PathBuf;
    use sunbeam_build::{files_in_dir_recursive_ending_with, SunbeamConfig};

    pub fn build_all_css() {
        let sunbeam_dir = sunbeam_dir();
        if sunbeam_dir.is_none() {
            return;
        }
        let sunbeam_dir = sunbeam_dir.unwrap();

        let files = files_in_dir_recursive_ending_with(views_dir(), ".rs");

        let config = include_str!("Sunbeam.yml");
        let config: SunbeamConfig = serde_yaml::from_str(config).unwrap();

        let css = sunbeam_build::parse_rust_files(files, &config).unwrap();

        let mut all_classes = vec![];

        for class in css.iter() {
            all_classes.push(&class.class_name);
        }
        let all_classes: HashSet<&String> = all_classes.into_iter().collect();

        let mut all_class_strings = "".to_string();

        for class in all_classes {
            all_class_strings += "const _: &'static str = css!(\"";
            all_class_strings += class;
            all_class_strings += "\");\n";
        }

        std::fs::write(sunbeam_dir.join("all_sunbeam_css.rs"), all_class_strings).unwrap();
    }

    fn views_dir() -> PathBuf {
        crate_dir().join("src/views")
    }

    fn crate_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn sunbeam_dir() -> Option<PathBuf> {
        std::env::var("SUNBEAM_DIR").ok().map(PathBuf::from)
    }
}
