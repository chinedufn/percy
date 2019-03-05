//! Note: Intentionally kept in it's own file for easy inclusion into The Percy Book

#![feature(proc_macro_hygiene)]

use router_rs::prelude::*;
use std::str::FromStr;
use virtual_dom_rs::prelude::*;

#[route(path = "/users/:id/favorite-meal/:meal")]
fn route_data_and_param(id: u16, state: Provided<SomeState>, meal: Meal) -> VirtualNode {
    let id = format!("{}", id);
    let meal = format!("{:#?}", meal);

    html! {
        <div> User { id } loves { meal } </div>
    }
}

#[test]
fn provided_data_and_param() {
    let mut router = Router::default();

    router.provide(SomeState { happy: true });

    router.set_route_handlers(create_routes![route_data_and_param]);

    assert_eq!(
        &router
            .view("/users/10/favorite-meal/breakfast")
            .unwrap()
            .to_string(),
        // TODO: This is a bug with our text implementation. Will fix...
        // We want this to be <div> User 10 loves breakfast </div>
        "<div>User10lovesBreakfast</div>"
    );
}

struct SomeState {
    happy: bool,
}

#[derive(Debug)]
enum Meal {
    Breakfast,
    Lunch,
    Dinner,
}

impl FromStr for Meal {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "breakfast" => Meal::Breakfast,
            "lunch" => Meal::Lunch,
            "dinner" => Meal::Dinner,
            _ => Err(())?,
        })
    }
}
