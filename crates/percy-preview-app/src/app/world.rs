mod message;

pub use self::message::*;
use std::ops::Deref;

mod world_config;
use self::resources::Resources;
use self::state::State;
pub(super) use self::world_config::WorldConfig;

mod resources;
mod state;

pub(crate) struct World {
    pub(crate) resources: Resources,
    pub(crate) state: State,
}

pub(super) fn create_world(config: WorldConfig) -> World {
    World {
        state: State {
            previews: config.previews,
        },
        resources: Resources {
            router: config.router,
        },
    }
}

impl Deref for World {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
