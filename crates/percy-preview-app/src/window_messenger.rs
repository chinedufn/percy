use crate::app::{Msg, World};
use app_world::AppWorldWrapper;

#[derive(Clone)]
pub(crate) struct WindowMessenger {
    world: AppWorldWrapper<World>,
}

impl WindowMessenger {
    pub fn new(world: AppWorldWrapper<World>) -> Self {
        Self { world }
    }

    pub fn msg_set_path(&self, new_path: String) {
        self.world.msg(Msg::SetPath(new_path))
    }
}
