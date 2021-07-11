# route macro

The `#[route(...)]` attribute macro is used to annotate functions that we cant to get called
when we visit a certain route.

Before diving into how it works, let's take a look at what code the macro generates for you.

Seeing the end result will make it easier to understand what we're doing and why we do it.

## Generated Code

Let's say that you have a file that looks like this:

```rust
// Imported from crates/percy-router-macro-test/src/book_example.rs

{{#include ../../../../../crates/percy-router-macro-test/src/book_example.rs}}
```

The `#[route(...)]` macro above will automatically generate the following code (some unimportant bits have been removed for brevity):

```rust
// TODO:: This code example isn't imported from a real file so it might go stale over time.

fn route_data_and_param(id: u16, state: Provided<SomeState>, meal: Meal) -> VirtualNode {
  // ... removed ...
}
fn create_route_data_and_param() -> Route {
    fn route_param_parser(param_key: &str, param_val: &str) -> Option<Box<dyn RouteParam>> {
        match param_key {
            "id" => {
                return Some(Box::new(
                    u16::from_str_param(param_val).expect("Macro parsed param"),
                ));
            }
            "meal" => {
                return Some(Box::new(
                    Meal::from_str_param(param_val).expect("Macro parsed param"),
                ));
            }
            _ => panic!("TODO: Handle this case..."),
        };
        None
    }
    Route::new(
        "/users/:id/favorite-meal/:meal",
        Box::new(route_param_parser),
    )
}
pub mod __route_data_and_param_mod__ {
    #![deny(warnings)]
    #![allow(non_camel_case_types)]
    use super::*;
    pub struct route_data_and_param_handler {
        route: Route,
        provided: Option<ProvidedMap>,
    }
    impl route_data_and_param_handler {
        pub fn new() -> route_data_and_param_handler {
            route_data_and_param_handler {
                route: create_route_data_and_param(),
                provided: None,
            }
        }
    }
    impl RouteHandler for route_data_and_param_handler {
        fn route(&self) -> &Route {
            &self.route
        }
        fn set_provided(&mut self, provided: ProvidedMap) {
            self.provided = Some(provided);
        }
        fn provided(&self) -> &ProvidedMap {
            &self.provided.as_ref().unwrap()
        }
        fn view(&self, incoming_route: &str) -> VirtualNode {
            let id = self
                .route()
                .find_route_param(incoming_route, "id")
                .expect("Finding route param");
            let meal = self
                .route()
                .find_route_param(incoming_route, "meal")
                .expect("Finding route param");
            let state = self.provided().borrow();
            let state = state
                .get(&std::any::TypeId::of::<Provided<SomeState>>())
                .unwrap()
                .downcast_ref::<Provided<SomeState>>()
                .expect("Downcast param");
            route_data_and_param(
                u16::from_str_param(id).expect(
                    // ... removed ...
                )),
                Provided::clone(state),
                Meal::from_str_param(meal).expect(
                  // ... removed ...
                ),
            )
        }
    }
}
fn provided_data_and_param() {
    let mut router = Router::default();
    router.provide(SomeState { happy: true });
    router.set_route_handlers(<[_]>::into_vec(box [Box::new(
        self::__route_data_and_param_mod__::route_data_and_param_handler::new(),
    )]));
    // ... removed ...
}
```
