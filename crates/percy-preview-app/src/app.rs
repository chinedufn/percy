use std::sync::{Arc, Mutex};

use app_world::AppWorldWrapper;

use crate::create_router;

pub(crate) use self::config::AppConfig;
pub use self::world::async_task_spawner;
use self::world::{create_world, WorldConfig};
pub(crate) use self::world::{Msg, World};

mod config;

mod world;

/// Powers an application that can be used to preview view components.
pub(crate) struct PercyPreviewApp {
    pub(crate) world: AppWorldWrapper<World>,
}

impl PercyPreviewApp {
    /// Create a new PercyPreviewApp.
    pub fn new(config: AppConfig, render: Arc<Mutex<Box<dyn FnMut() -> ()>>>) -> Self {
        let router = create_router();

        let world = create_world(WorldConfig {
            async_task_spawner: config.async_task_spawner,
            after_path_change: config.after_path_change,
            previews: config.previews,
            render,
            router,
        });
        let world = AppWorldWrapper::new(world);

        world.msg(Msg::AttachRouteDataProvider);

        Self { world }
    }
}
