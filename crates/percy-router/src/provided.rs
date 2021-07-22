use crate::router::Router;
use std::any::TypeId;
use std::ops::Deref;
use std::rc::Rc;

/// Data that was provided by the developer.
///
/// ```ignore
/// struct State {
///     count: u8
/// }
///
/// #[route(path = "/")]
/// fn route_provided_data(state: Provided<State>) -> VirtualNode {
///     VirtualNode::Text(format!("Count: {}", state.count).into())
/// }
///
/// fn main () {
///     let mut router = Router::new(vec![]);
///     router.provide(State {count: 50});
/// }
/// ```
pub struct Provided<T> {
    /// The application data to provide to a route.
    pub data: Rc<T>,
}

impl<T> Deref for Provided<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Clone for Provided<T> {
    fn clone(&self) -> Self {
        Provided {
            data: Rc::clone(&self.data),
        }
    }
}

impl Router {
    /// Provide the application state data that different routes need.
    pub fn provide<T: 'static>(&mut self, provided: T) {
        let provided = Provided {
            data: Rc::new(provided),
        };

        let type_id = TypeId::of::<Provided<T>>();

        let provided = Box::new(provided);

        self.provided.borrow_mut().insert(type_id, provided);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct State {
        count: u8,
    }

    #[test]
    fn provide() {
        let mut router = Router::new(vec![]);
        router.provide(State { count: 50 });

        let state = router.provided.borrow();
        let state = state
            .get(&TypeId::of::<Provided<State>>())
            .unwrap()
            .downcast_ref::<Provided<State>>()
            .expect("Downcast state");

        assert_eq!(state.count, 50);
    }
}
