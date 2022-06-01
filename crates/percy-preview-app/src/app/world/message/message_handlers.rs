use super::Msg;
use crate::app::World;
use app_world::AppWorld;
use app_world::AppWorldWrapper;

mod set_route_data_provider_message_handler;

impl AppWorld for World {
    type Message = Msg;

    fn msg(&mut self, message: Self::Message, wrap: AppWorldWrapper<Self>) {
        match message {
            Msg::ProvideRouteData => self.set_route_data_provider_message_handler(wrap),
        }
    }
}
