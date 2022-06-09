mod message_handlers;

/// Used to tell the World to handle some occurrence.
///
/// For example, a Msg might get sent when a button is clicked.
pub enum Msg {
    /// Attach the route data provider to the router.
    AttachRouteDataProvider,
    /// Set the active URL path, such as "/some/path".
    SetPath(String),
}
