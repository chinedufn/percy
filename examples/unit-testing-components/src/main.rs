#[macro_use]
extern crate virtual_dom_rs;

use virtual_dom_rs::virtual_node::VirtualNode;

fn main() {
    println!("To see this example in action:");
    println!("cargo test -p unit-testing-components");
}

#[allow(unused)]
fn water_bottle_view(percent_full: f32) -> VirtualNode {
    if percent_full > 0.5 {
        full_water_bottle()
    } else {
        struggling_water_bottle(percent_full)
    }
}

#[allow(unused)]
fn full_water_bottle() -> VirtualNode {
    html! {
        <div> <span label="full-water",>{ "I am full of delicous and refreshing H20!"}</span> </div>
    }
}

#[allow(unused)]
fn struggling_water_bottle(percent_full: f32) -> VirtualNode {
    let message = format!(
        "Please fill me up :( I am only {} percent full :(",
        percent_full
    );

    html! {
        <div label="struggle-water",> { message } </div>
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

        let struggle_water = water_bottle_view(0.2587);

        assert_eq!(
            struggle_water.children.as_ref().unwrap()[0]
                .text
                .as_ref()
                .unwrap(),
            "Please fill me up :( I am only 0.2587 percent full :("
        )
    }
}
