#![feature(proc_macro_hygiene)]

use virtual_dom_rs::prelude::*;

fn main() {
    println!("To see this example in action:");
    println!("cargo test -p unit-testing-components");
}

#[allow(unused)]
fn full_water_bottle() -> VirtualNode {
    html! {
    <div>
        <span label="full-water">
          I am full of delicious and refreshing H20!
        </span>
    </div>
    }
}

#[allow(unused)]
fn struggling_water_bottle(percent_full: f32) -> VirtualNode {
    let message = format!(
        "Please fill me up :( I am only {} percent full :(",
        percent_full
    );
    let message = VirtualNode::text(&*message);

    html! {
        <div label="struggle-water">
         { message }
        </div>
    }
}

#[allow(unused)]
fn water_bottle_view(percent_full: f32) -> VirtualNode {
    if percent_full > 0.5 {
        full_water_bottle()
    } else {
        struggling_water_bottle(percent_full)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditional_water_messaging() {
        assert_eq!(
            water_bottle_view(0.7)
                .filter_label_equals("full-water")
                .len(),
            1
        );

        let water_view = water_bottle_view(0.2587);

        // FIXME: Change this back to one after handling our whitespace issue
        assert_eq!(
            water_view
                .as_velement_ref()
                .expect("Not an element node")
                .children[1]
                .as_vtext_ref()
                .expect("Not a text node")
                .text,
            "Please fill me up :( I am only 0.2587 percent full :("
        )
    }
}
