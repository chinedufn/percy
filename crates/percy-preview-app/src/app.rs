use std::sync::{Arc, Mutex};

use crate::create_router;
use app_world::AppWorldWrapper;

mod config;

pub use self::config::AppConfig;

mod world;
pub(crate) use self::world::World;
use self::world::{create_world, Msg, WorldConfig};

/// Powers an application that can be used to preview view components.
pub(crate) struct PercyPreviewApp {
    pub(crate) world: AppWorldWrapper<World>,
}

impl PercyPreviewApp {
    /// Create a new PercyPreviewApp.
    pub fn new(config: AppConfig, render: Arc<Mutex<Box<dyn FnMut() -> ()>>>) -> Self {
        let router = create_router();

        let world = create_world(WorldConfig {
            previews: config.previews,
            render,
            router,
        });
        let world = AppWorldWrapper::new(world);

        world.msg(Msg::ProvideRouteData);

        Self { world }
    }
}
