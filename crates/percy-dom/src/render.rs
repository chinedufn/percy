//! Utilities to help with rendering.

use crate::{PercyDom, VirtualNode};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::JsCast;

/// Given a [`PercyDom`] and a function that renders a [`VirtualNode`],
/// return a function that can call that render function up to once per browser
/// animation frame.
///
/// The returned `VirtualNode` is used to update the `PercyDom` instance.
///
/// After the first call to the returned render function, further calls will be
/// ignored until the next render, at after which you will be able to schedule
/// another render again.
///
/// This is useful for when your application state changes and you want to schedule
/// a re-render to occur on the next animation frame.
///
/// # Example
///
/// ```
/// # use percy_dom::{
/// #    prelude::*,
/// #    render::create_render_scheduler
/// # };
///
/// struct MyApp {
///     counter: u8
/// }
///
/// impl MyApp {
///     fn render(&self) -> VirtualNode {
///         html! { <div>Count: { self.counter }</div> }
///     }
/// }
///
/// fn start () {
///     let app = MyApp { counter: 5 };
///     let pdom = make_percy_dom_somehow();
///
///     let mut render = create_render_scheduler(
///         pdom,
///         move || {
///             app.render()
///         }
///     );
///
///     // In a real application you might call this whenever your
///     // application state changes.
///     render();
/// }
///
/// # fn make_percy_dom_somehow() -> PercyDom { unimplemented!() }
/// ```
pub fn create_render_scheduler<F: FnMut() -> VirtualNode + 'static>(
    mut percy_dom: PercyDom,
    mut render: F,
) -> Box<dyn FnMut()> {
    let render_is_scheduled = Rc::new(Cell::new(false));
    let render_is_scheduled_clone = render_is_scheduled.clone();

    let render = move || {
        render_is_scheduled.set(false);

        let vdom = render();
        percy_dom.update(vdom);
    };

    let render = Box::new(render) as Box<dyn FnMut()>;
    let render = Closure::wrap(render);

    let window = web_sys::window().unwrap();
    let render_raf = move || {
        if render_is_scheduled_clone.get() {
            return;
        }

        render_is_scheduled_clone.set(true);

        window
            .request_animation_frame(render.as_ref().as_ref().unchecked_ref())
            .unwrap();
    };

    Box::new(render_raf) as Box<dyn FnMut() -> ()>
}
