#[cfg(not(target_arch = "wasm32"))]
pub use self::not_wasm::*;
#[cfg(target_arch = "wasm32")]
pub use self::wasm::*;

#[cfg(target_arch = "wasm32")]
mod wasm {
    pub type DomMouseEvent = web_sys::MouseEvent;
}

#[cfg(not(target_arch = "wasm32"))]
mod not_wasm {

    pub type DomMouseEvent = SimulatedMouseEvent;

    /// A fake MouseEvent, useful for testing.
    #[cfg(not(target_arch = "wasm32"))]
    pub struct SimulatedMouseEvent {}

    impl SimulatedMouseEvent {
        /// Create a new fake input event.
        pub fn new() -> Self {
            SimulatedMouseEvent {}
        }
    }
}
