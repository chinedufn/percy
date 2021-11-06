use crate::EventAttribFn;
use std::borrow::Cow;
use std::rc::Rc;
use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::JsValue;

/// A specially supported attributes.
#[derive(Default)]
pub struct SpecialAttributes {
    /// A function that gets called when the virtual node is first turned into a real node, or when
    /// this virtual node's attribute's replace some other existing node with the same tag.
    ///
    /// The on_create_elem attribute has a unique identifier so that when we patch over another
    /// DOM element that also has an `on_create_elem` we know that we still need to call
    /// `on_create_elem`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use virtual_node::VirtualNode;
    /// use virtual_node::wrap_closure;
    ///
    /// let mut node = VirtualNode::element("div");
    /// let on_create_elem =move |_elem: web_sys::Element| {
    /// };
    ///
    /// node.as_velement_mut().unwrap().special_attributes.on_create_elem =
    ///     Some(("some-unique-key".into(), wrap_closure(on_create_elem)));
    /// ```
    pub on_create_elem: Option<(Cow<'static, str>, EventAttribFn)>,
    /// Allows setting the innerHTML of an element. Be sure to escape all untrusted input to avoid
    /// cross site scripting attacks.
    pub dangerous_inner_html: Option<String>,
}

impl SpecialAttributes {
    /// If there is an `on_create_elem` function defined, call it.
    pub fn maybe_call_on_create_elem(&self, element: &web_sys::Element) {
        #[cfg(target_arch = "wasm32")]
        if let Some(on_create_elem) = &self.on_create_elem {
            use wasm_bindgen::JsCast;

            let on_create_elem: &js_sys::Function =
                on_create_elem.1.as_ref().as_ref().unchecked_ref();
            on_create_elem
                .call1(&wasm_bindgen::JsValue::NULL, element)
                .unwrap();
        }

        let _ = element;
    }
}

impl PartialEq for SpecialAttributes {
    fn eq(&self, rhs: &Self) -> bool {
        self.dangerous_inner_html == rhs.dangerous_inner_html
    }
}

/// Wraps a function as a wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
pub fn wrap_closure<F: FnMut(T) + 'static, T: FromWasmAbi + 'static>(func: F) -> EventAttribFn {
    use wasm_bindgen::closure::Closure;

    let closure = Closure::wrap(Box::new(func) as Box<dyn FnMut(_)>);
    let closure_rc = std::rc::Rc::new(closure);

    EventAttribFn(closure_rc)
}

#[cfg(not(target_arch = "wasm32"))]
pub fn wrap_closure<F: FnMut(T) + 'static, T: FromWasmAbi + 'static>(_func: F) -> EventAttribFn {
    EventAttribFn(Rc::new(JsValue::NULL))
}
