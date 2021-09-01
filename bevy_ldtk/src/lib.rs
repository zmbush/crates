use bevy::prelude::*;

mod assets;
mod render;

pub use assets::LdtkProject;
pub use render::{AttachEnumsEvent, EntitySpawn, LdtkProjectBundle, LdtkProjectCfg, LdtkRenderType};

#[derive(Debug)]
pub struct LdtkPlugin;

pub static LDTK_RENDER: &str = "ldtk_render";
pub static LDTK_CLEANUP: &str = "ldtk_cleanup";
pub static LDTK_HOT_RELOAD: &str = "ldtk_hot_reload";

impl Plugin for LdtkPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<assets::LdtkProject>()
            .init_asset_loader::<assets::LdtkProjectLoader>()
            .add_event::<render::EntitySpawn>()
            .add_event::<render::AttachEnumsEvent>()
            .add_system(render::render_ldtk_projects.system().label(LDTK_RENDER))
            .add_system(
                render::ldtk_entity_cleanup
                    .system()
                    .label(LDTK_CLEANUP)
                    .before(LDTK_RENDER),
            )
            .add_system(
                render::ldtk_hot_reload
                    .system()
                    .label(LDTK_HOT_RELOAD)
                    .before(LDTK_CLEANUP),
            );
    }
}
