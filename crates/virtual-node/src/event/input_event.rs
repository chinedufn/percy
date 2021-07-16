#[cfg(not(target_arch = "wasm32"))]
pub use self::not_wasm::*;
#[cfg(target_arch = "wasm32")]
pub use self::wasm::*;

#[cfg(target_arch = "wasm32")]
mod wasm {
    pub type DomInputEvent = web_sys::InputEvent;
}

#[cfg(not(target_arch = "wasm32"))]
mod not_wasm {

    pub type DomInputEvent = SimulatedInputEvent;

    /// A fake InputEvent, useful for testing.
    #[cfg(not(target_arch = "wasm32"))]
    pub struct SimulatedInputEvent {
        data: Option<String>,
    }

    impl SimulatedInputEvent {
        /// Create a new fake input event.
        pub fn new(data: Option<String>) -> Self {
            SimulatedInputEvent { data }
        }

        /// Get the text from the input event.
        pub fn data(&self) -> Option<String> {
            self.data.clone()
        }

        /// Set the data for the input event.
        pub fn set_data(&mut self, data: Option<String>) {
            self.data = data;
        }
    }
}
