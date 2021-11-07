use std::borrow::Cow;
use std::cell::RefCell;
use std::ops::DerefMut;

/// A specially supported attributes.
#[derive(Default, PartialEq)]
pub struct SpecialAttributes {
    /// A a function that gets called when the virtual node is first turned into a real node.
    ///
    /// See [`SpecialAttributes.set_on_create_element`] for more documentation.
    on_create_element: Option<KeyAndElementFn>,
    /// A a function that gets called when the virtual node is first turned into a real node.
    ///
    /// See [`SpecialAttributes.set_on_remove_element`] for more documentation.
    on_remove_element: Option<KeyAndElementFn>,
    /// Allows setting the innerHTML of an element.
    ///
    /// # Danger
    ///
    /// Be sure to escape all untrusted input to avoid cross site scripting attacks.
    pub dangerous_inner_html: Option<String>,
}

impl SpecialAttributes {
    /// The key for the on create element function
    pub fn on_create_element_key(&self) -> Option<&Cow<'static, str>> {
        self.on_create_element.as_ref().map(|k| &k.key)
    }

    /// Set the [`SpecialAttributes.on_create_element`] function.
    ///
    /// # Key
    ///
    /// The key is used when one virtual-node is being patched over another.
    ///
    /// If the new node's key is different from the old node's key, the on create element function
    /// gets called.
    ///
    /// If the keys are the same, the function does not get called.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use virtual_node::VirtualNode;
    /// use wasm_bindgen::JsValue;
    ///
    /// let mut node = VirtualNode::element("div");
    ///
    /// // A key can be any `Into<Cow<'static, str>>`.
    /// let key = "some-key";
    ///
    /// let on_create_elem = move |elem: web_sys::Element| {
    ///     assert_eq!(elem.id(), "");
    /// };
    ///
    /// node
    ///     .as_velement_mut()
    ///     .unwrap()
    ///     .special_attributes
    ///     .set_on_create_element(key, on_create_elem);
    /// ```
    pub fn set_on_create_element<Key, Func>(&mut self, key: Key, func: Func)
    where
        Key: Into<Cow<'static, str>>,
        Func: FnMut(web_sys::Element) + 'static,
    {
        self.on_create_element = Some(KeyAndElementFn {
            key: key.into(),
            func: RefCell::new(ElementFunc::OneArg(Box::new(func))),
        });
    }

    // Used by the html-macro
    #[doc(hidden)]
    pub fn set_on_create_element_no_args<Key, Func>(&mut self, key: Key, func: Func)
    where
        Key: Into<Cow<'static, str>>,
        Func: FnMut() + 'static,
    {
        self.on_create_element = Some(KeyAndElementFn {
            key: key.into(),
            func: RefCell::new(ElementFunc::NoArgs(Box::new(func))),
        });
    }

    /// If an `on_create_element` function was set, call it.
    pub fn maybe_call_on_create_element(&self, element: &web_sys::Element) {
        if let Some(on_create_elem) = &self.on_create_element {
            on_create_elem.call(element.clone());
        }

        let _ = element;
    }
}

impl SpecialAttributes {
    /// The key for the on remove element function
    pub fn on_remove_element_key(&self) -> Option<&Cow<'static, str>> {
        self.on_remove_element.as_ref().map(|k| &k.key)
    }

    /// Set the [`SpecialAttributes.on_remove_element`] function.
    ///
    /// # Key
    ///
    /// The key is used when one virtual-node is being patched over another.
    ///
    /// If the old node's key is different from the new node's key, the on remove element function
    /// gets called for the old element.
    ///
    /// If the keys are the same, the function does not get called.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use virtual_node::VirtualNode;
    /// use wasm_bindgen::JsValue;
    ///
    /// let mut node = VirtualNode::element("div");
    ///
    /// // A key can be any `Into<Cow<'static, str>>`.
    /// let key = "some-key";
    ///
    /// let on_remove_elem = move |elem: web_sys::Element| {
    ///     assert_eq!(elem.id(), "");
    /// };
    ///
    /// node
    ///     .as_velement_mut()
    ///     .unwrap()
    ///     .special_attributes
    ///     .set_on_remove_element(key, on_remove_elem);
    /// ```
    pub fn set_on_remove_element<Key, Func>(&mut self, key: Key, func: Func)
    where
        Key: Into<Cow<'static, str>>,
        Func: FnMut(web_sys::Element) + 'static,
    {
        self.on_remove_element = Some(KeyAndElementFn {
            key: key.into(),
            func: RefCell::new(ElementFunc::OneArg(Box::new(func))),
        });
    }

    // Used by the html-macro
    #[doc(hidden)]
    pub fn set_on_remove_element_no_args<Key, Func>(&mut self, key: Key, func: Func)
    where
        Key: Into<Cow<'static, str>>,
        Func: FnMut() + 'static,
    {
        self.on_remove_element = Some(KeyAndElementFn {
            key: key.into(),
            func: RefCell::new(ElementFunc::NoArgs(Box::new(func))),
        });
    }

    /// If an `on_remove_element` function was set, call it.
    pub fn maybe_call_on_remove_element(&self, element: &web_sys::Element) {
        if let Some(on_remove_elem) = &self.on_remove_element {
            on_remove_elem.call(element.clone());
        }

        let _ = element;
    }
}

struct KeyAndElementFn {
    key: Cow<'static, str>,
    func: RefCell<ElementFunc>,
}

enum ElementFunc {
    NoArgs(Box<dyn FnMut()>),
    OneArg(Box<dyn FnMut(web_sys::Element)>),
}

impl KeyAndElementFn {
    fn call(&self, element: web_sys::Element) {
        match self.func.borrow_mut().deref_mut() {
            ElementFunc::NoArgs(func) => (func)(),
            ElementFunc::OneArg(func) => (func)(element),
        };
    }
}

impl PartialEq for KeyAndElementFn {
    fn eq(&self, rhs: &Self) -> bool {
        self.key == rhs.key
    }
}
