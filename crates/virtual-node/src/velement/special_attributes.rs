use crate::event::RealDom;
use std::borrow::Cow;
use std::cell::RefCell;

/// A specially supported attributes.
pub struct SpecialAttributes<Dom: RealDom> {
    key: Option<Cow<'static, str>>,
    /// A function that gets called when the virtual node is first turned into a real node.
    ///
    /// See [`SpecialAttributes.set_on_create_element`] for more documentation.
    on_create_element: Option<CreateOrRemoveElementFn<Dom>>,
    /// A function that gets called when the virtual node is first turned into a real node.
    ///
    /// See [`SpecialAttributes.set_on_remove_element`] for more documentation.
    on_remove_element: Option<CreateOrRemoveElementFn<Dom>>,
    /// Allows setting the innerHTML of an element.
    ///
    /// # Danger
    ///
    /// Be sure to escape all untrusted input to avoid cross site scripting attacks.
    pub dangerous_inner_html: Option<String>,
}

/// An error when attempting to perform an action that requires the [`SpecialAttributes::key`] to be
/// set.
#[derive(Debug)]
pub struct KeyNotSetError;

impl<Dom: RealDom> SpecialAttributes<Dom> {
    /// Keys can distinguish two elements that have the same tag.
    ///
    /// For example, if one `div` has key `old-key`, and another `div` has key `new-key`, the two
    /// elements are considered different.
    pub fn key(&self) -> Option<&str> {
        self.key.as_ref().map(|key| key.as_ref())
    }

    /// Set the element's `key`.
    pub fn set_key<Key>(&mut self, key: Key)
    where
        Key: Into<Cow<'static, str>>,
    {
        self.key = Some(key.into());
    }

    /// Returns the element's key if an `on_create_element` function is set.
    pub fn on_create_element_key(&self) -> Option<&Cow<'static, str>> {
        if self.on_create_element.is_some() {
            return self.key.as_ref();
        }
        None
    }

    /// Combines [`SpecialAttributes::set_key`] and [`SpecialAttributes::set_on_create_element`].
    pub fn set_key_and_on_create_element<Key, Func>(&mut self, key: Key, func: Func)
    where
        Key: Into<Cow<'static, str>>,
        Func: FnMut(Dom::Element) + 'static,
    {
        self.set_key(key);
        self.set_on_create_element(func).unwrap()
    }

    /// Set the [`SpecialAttributes.on_create_element`] function.
    ///
    /// # Key
    ///
    /// The [`SpecialAttributes::key`] is used when one virtual-node is being patched over another.
    ///
    /// If the new node's key is different from the old node's key, the on create element function
    /// gets called.
    ///
    /// If the keys are the same, the function does not get called.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use virtual_node::VirtualNodeWebSys;
    /// use wasm_bindgen::JsValue;
    ///
    /// let mut node = VirtualNodeWebSys::new_element("div");
    ///
    /// // A key can be any `Into<Cow<'static, str>>`.
    /// let key = "some-key";
    ///
    /// let on_create_elem = move |elem: web_sys::Element| {
    ///     assert_eq!(elem.id(), "");
    /// };
    ///
    /// let elem = node.as_elem_mut().unwrap();
    /// elem.special_attributes.set_key(key);
    /// elem.special_attributes.set_on_create_element(on_create_elem).unwrap();
    /// ```
    pub fn set_on_create_element<Func>(&mut self, func: Func) -> Result<(), KeyNotSetError>
    where
        Func: FnMut(Dom::Element) + 'static,
    {
        if self.key.is_none() {
            return Err(KeyNotSetError);
        }

        self.on_create_element = Some(CreateOrRemoveElementFn {
            func: RefCell::new(ElementFunc::OneArg(Box::new(func))),
        });
        Ok(())
    }

    // Used by the html-macro
    #[doc(hidden)]
    pub fn set_on_create_element_no_args<Func>(&mut self, func: Func) -> Result<(), KeyNotSetError>
    where
        Func: FnMut() + 'static,
    {
        if self.key.is_none() {
            return Err(KeyNotSetError);
        }

        self.on_create_element = Some(CreateOrRemoveElementFn {
            func: RefCell::new(ElementFunc::NoArgs(Box::new(func))),
        });
        Ok(())
    }

    /// If an `on_create_element` function was set, call it.
    pub fn maybe_call_on_create_element(&self, element: &Dom::Element) {
        if let Some(on_create_elem) = &self.on_create_element {
            on_create_elem.call(element.clone());
        }
    }
}

impl<Dom: RealDom> SpecialAttributes<Dom> {
    /// Returns the element's key if an `on_remove_element` function is set.
    pub fn on_remove_element_key(&self) -> Option<&Cow<'static, str>> {
        if self.on_remove_element.is_some() {
            return self.key.as_ref();
        }
        None
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
    /// # use virtual_node::VirtualNodeWebSys;
    /// use wasm_bindgen::JsValue;
    ///
    /// let mut node = VirtualNodeWebSys::new_element("div");
    ///
    /// // A key can be any `Into<Cow<'static, str>>`.
    /// let key = "some-key";
    ///
    /// let on_remove_elem = move |elem: web_sys::Element| {
    ///     assert_eq!(elem.id(), "");
    /// };
    ///
    /// let elem = node.as_elem_mut().unwrap();
    /// elem.special_attributes.set_key(key);
    /// elem.special_attributes.set_on_remove_element(on_remove_elem);
    /// ```
    pub fn set_on_remove_element<Func>(&mut self, func: Func) -> Result<(), KeyNotSetError>
    where
        Func: FnMut(Dom::Element) + 'static,
    {
        if self.key.is_none() {
            return Err(KeyNotSetError);
        }

        self.on_remove_element = Some(CreateOrRemoveElementFn {
            func: std::cell::RefCell::new(ElementFunc::OneArg(Box::new(func))),
        });
        Ok(())
    }

    // Used by the html-macro
    #[doc(hidden)]
    pub fn set_on_remove_element_no_args<Func>(&mut self, func: Func) -> Result<(), KeyNotSetError>
    where
        Func: FnMut() + 'static,
    {
        if self.key.is_none() {
            return Err(KeyNotSetError);
        }

        self.on_remove_element = Some(CreateOrRemoveElementFn {
            func: RefCell::new(ElementFunc::NoArgs(Box::new(func))),
        });
        Ok(())
    }

    /// If an `on_remove_element` function was set, call it.
    pub fn maybe_call_on_remove_element(&self, element: &Dom::Element) {
        if let Some(on_remove_elem) = &self.on_remove_element {
            on_remove_elem.call(element.clone());
        }

        let _ = element;
    }

    pub(crate) fn map_dom<New: RealDom>(
        self,
        map: &dyn Fn(Box<dyn FnMut(Dom::Element)>) -> Box<dyn FnMut(New::Element)>,
    ) -> SpecialAttributes<New> {
        SpecialAttributes {
            key: self.key,
            on_create_element: self.on_create_element.map(|func| func.map_dom(map)),
            on_remove_element: self.on_remove_element.map(|func| func.map_dom(map)),
            dangerous_inner_html: self.dangerous_inner_html,
        }
    }
}

struct CreateOrRemoveElementFn<Dom: RealDom> {
    func: RefCell<ElementFunc<Dom>>,
}

enum ElementFunc<Dom: RealDom> {
    NoArgs(Box<dyn FnMut()>),
    OneArg(Box<dyn FnMut(Dom::Element)>),
}

impl<Dom: RealDom> CreateOrRemoveElementFn<Dom> {
    fn call(&self, element: Dom::Element) {
        use std::ops::DerefMut;

        match self.func.borrow_mut().deref_mut() {
            ElementFunc::NoArgs(func) => func(),
            ElementFunc::OneArg(func) => func(element),
        };
    }

    fn map_dom<New: RealDom>(
        self,
        map: &dyn Fn(Box<dyn FnMut(Dom::Element)>) -> Box<dyn FnMut(New::Element)>,
    ) -> CreateOrRemoveElementFn<New> {
        let func = self.func.into_inner();
        match func {
            ElementFunc::NoArgs(func) => CreateOrRemoveElementFn {
                func: RefCell::new(ElementFunc::NoArgs(func)),
            },
            ElementFunc::OneArg(func) => {
                let func = map(func);
                CreateOrRemoveElementFn {
                    func: RefCell::new(ElementFunc::OneArg(func)),
                }
            }
        }
    }
}

impl<Dom: RealDom> PartialEq for CreateOrRemoveElementFn<Dom> {
    fn eq(&self, rhs: &Self) -> bool {
        let _ = rhs;
        // TODO: Arbitrarily chosen
        true
    }
}

impl<Dom: RealDom> Default for SpecialAttributes<Dom> {
    fn default() -> Self {
        Self {
            key: None,
            on_create_element: None,
            on_remove_element: None,
            dangerous_inner_html: None,
        }
    }
}

impl<Dom: RealDom> PartialEq for SpecialAttributes<Dom> {
    fn eq(&self, other: &Self) -> bool {
        let SpecialAttributes {
            key: key_lhs,
            on_create_element: on_create_element_lhs,
            on_remove_element: on_remove_element_lhs,
            dangerous_inner_html: dangerous_inner_html_lhs,
        } = self;
        let SpecialAttributes {
            key: key_rhs,
            on_create_element: on_create_element_rhs,
            on_remove_element: on_remove_element_rhs,
            dangerous_inner_html: dangerous_inner_html_rhs,
        } = other;

        key_lhs == key_rhs
            && on_create_element_lhs == on_create_element_rhs
            && on_remove_element_lhs == on_remove_element_rhs
            && dangerous_inner_html_lhs == dangerous_inner_html_rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VirtualElement;

    /// Verify that we cannot set `on_create_element` if no `key` has been set.
    #[test]
    fn error_setting_on_create_element_if_no_key() {
        let mut elem = create_div();

        let err1 = elem
            .special_attributes
            .set_on_create_element(|_| {})
            .err()
            .unwrap();
        let err2 = elem
            .special_attributes
            .set_on_create_element_no_args(|| {})
            .err()
            .unwrap();

        assert!(matches!(err1, KeyNotSetError));
        assert!(matches!(err2, KeyNotSetError));
    }

    /// Verify that we cannot set `on_remove_element` if no `key` has been set.
    #[test]
    fn error_setting_on_remove_element_if_no_key() {
        let mut elem = create_div();
        let special = &mut elem.special_attributes;

        let err1 = special.set_on_remove_element(|_| {}).err().unwrap();
        let err2 = special.set_on_remove_element_no_args(|| {}).err().unwrap();

        assert!(matches!(err1, KeyNotSetError));
        assert!(matches!(err2, KeyNotSetError));
    }

    /// Verify that [`SpecialAttributes::on_create_element_key`] only returns the key if
    /// `on_create_element` is set.
    #[test]
    fn on_create_element_key() {
        let mut elem = create_div();
        let special = &mut elem.special_attributes;

        special.set_key("hello");
        assert_eq!(special.on_create_element_key(), None);

        special.set_on_create_element(|_| {}).unwrap();
        assert_eq!(
            special.on_create_element_key().map(|key| key.as_ref()),
            Some("hello")
        );
    }

    /// Verify that [`SpecialAttributes::maybe_call_on_remove_element`] only returns the key if
    /// `on_remove_element` is set.
    #[test]
    fn on_remove_element_key() {
        let mut elem = create_div();
        let special = &mut elem.special_attributes;

        special.set_key("hello");
        assert_eq!(special.on_remove_element_key(), None);

        special.set_on_remove_element(|_| {}).unwrap();
        assert_eq!(
            special.on_remove_element_key().map(|key| key.as_ref()),
            Some("hello")
        );
    }

    fn create_div() -> VirtualElement<()> {
        VirtualElement::new("div")
    }
}
