mod message_handlers;

/// Used to tell the World to handle some occurrence.
///
/// For example, a Msg might get sent when a button is clicked.
pub enum Msg {
    ProvideRouteData,
}
