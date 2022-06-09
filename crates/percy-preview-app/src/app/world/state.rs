use percy_preview::Preview;

/// Application state.
pub(crate) struct State {
    pub rendering_enabled: bool,
    pub active_path: String,
    pub previews: Vec<Preview>,
}
