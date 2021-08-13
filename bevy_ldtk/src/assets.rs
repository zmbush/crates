use std::collections::HashMap;

use bevy::{
    asset::{AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
};

#[derive(TypeUuid)]
#[uuid = "b5114809-ec1c-48ca-84c1-95f446050cee"]
pub struct LdtkProject {
    pub project: ldtk::Project,

    pub spritesheets: HashMap<i64, Handle<TextureAtlas>>,
    pub int_grid_colors: HashMap<i64, Vec<Handle<ColorMaterial>>>,
    pub entity_colors: HashMap<i64, Handle<ColorMaterial>>,
}

impl LdtkProject {
    fn new(project: ldtk::Project) -> Self {
        Self {
            project,

            spritesheets: HashMap::default(),
            int_grid_colors: HashMap::default(),
            entity_colors: HashMap::default(),
        }
    }
}

#[derive(Debug, Default)]
pub struct LdtkProjectLoader;
impl AssetLoader for LdtkProjectLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let project: ldtk::Project = serde_json::from_slice(bytes)?;
            let mut asset = LdtkProject::new(project);

            for tileset in &asset.project.defs.tilesets {
                let path = load_context.path().parent().unwrap().join(&tileset.rel_path);

                let asset_path = AssetPath::new(path, None);

                let texture_atlas: Handle<TextureAtlas> = load_context.set_labeled_asset(
                    &format!("Tileset {}", tileset.identifier),
                    LoadedAsset::new(TextureAtlas::from_grid(
                        load_context.get_handle(asset_path.clone()),
                        Vec2::new(tileset.tile_grid_size as f32, tileset.tile_grid_size as f32),
                        (tileset.px_wid / tileset.tile_grid_size) as usize,
                        (tileset.px_hei / tileset.tile_grid_size) as usize,
                    ))
                    .with_dependency(asset_path),
                );
                asset.spritesheets.insert(tileset.uid, texture_atlas);
            }

            for layer in asset
                .project
                .defs
                .layers
                .iter()
                .filter(|f| matches!(f.purple_type, ldtk::Type::IntGrid))
            {
                let mut colors = Vec::new();
                for (ix, i) in layer.int_grid_values.iter().enumerate() {
                    let clr =
                        Color::hex(&i.color[1..]).map_err(|e| anyhow::format_err!("Failed to parse color: {:?}", e))?;
                    let clr_mat = load_context.set_labeled_asset(
                        &format!(
                            "Layer Color {}x{}",
                            layer.identifier,
                            i.identifier.clone().unwrap_or(format!("#{}", ix))
                        ),
                        LoadedAsset::new(ColorMaterial::from(clr)),
                    );
                    colors.push(clr_mat);
                }
                asset.int_grid_colors.insert(layer.uid, colors);
            }

            for entity in &asset.project.defs.entities {
                let clr = Color::hex(&entity.color[1..])
                    .map_err(|e| anyhow::format_err!("Failed to parse color: {:?}", e))?;
                let clr_mat = load_context.set_labeled_asset(
                    &format!("Entity Color {}", entity.identifier,),
                    LoadedAsset::new(ColorMaterial::from(clr)),
                );
                asset.entity_colors.insert(entity.uid, clr_mat);
            }

            load_context.set_default_asset(LoadedAsset::new(asset));

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["ldtk"]
    }
}
