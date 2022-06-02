mod message;
pub use self::message::*;

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
        state: State {},
        resources: Resources {
            router: config.router,
        },
    }
}
