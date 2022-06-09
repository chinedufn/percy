mod message;

pub use self::message::*;
use std::ops::Deref;

mod world_config;
pub use self::resources::async_task_spawner;
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
            rendering_enabled: true,
            active_path: "/".to_string(),
            previews: config.previews,
        },
        resources: Resources {
            after_path_change: config.after_path_change,
            render_fn: config.render,
            router: config.router,
            async_task_spawner: config.async_task_spawner,
        },
    }
}

impl Deref for World {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
