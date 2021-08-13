use crate::LdtkProject;
use bevy::{ecs::system::EntityCommands, prelude::*};
use ldtk::{EntityInstance, TileInstance};

#[derive(Clone, Default, Debug)]
pub struct LdtkRenderedProject(Handle<LdtkProject>);

#[derive(Bundle, Clone, Default, Debug)]
pub struct LdtkRenderBundle {
    project: LdtkRenderedProject,

    transform: Transform,
    global_transform: GlobalTransform,
}

#[non_exhaustive]
pub struct AttachEnumsData<'a> {
    project: &'a LdtkProject,
    tileset_uid: i64,
    tile: &'a TileInstance,
}

type AttachEnumsFunction = Box<dyn Fn(&mut EntityCommands, AttachEnumsData) + Send + Sync + 'static>;
#[derive(Default)]
pub struct LdtkProjectCfg {
    pub render_type: LdtkRenderType,
    pub attach_enums: Option<AttachEnumsFunction>,
    pub rendered: Option<usize>,
}

#[derive(Copy, Clone, Debug)]
pub enum LdtkRenderType {
    SingleLevel(usize),
    FullWorld,
}

impl Default for LdtkRenderType {
    fn default() -> Self {
        LdtkRenderType::FullWorld
    }
}

#[derive(Bundle, Default)]
pub struct LdtkProjectBundle {
    pub ldtk_project: Handle<LdtkProject>,
    pub cfg: LdtkProjectCfg,

    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Copy, Clone)]
struct LevelInfo {
    world_x: i32,
    world_y: i32,
}

#[derive(Clone, Copy)]
struct LayerInfo {
    grid_width: i32,
    _grid_height: i32,
    grid_cell_size: i32,
    z_index: i32,
}

#[derive(Copy, Clone, Default)]
struct ExtraEntDefs {
    __tile_id: i32,
    __width: i32,
    __height: i32,
    __scale: f32,
}

pub fn ldtk_hot_reload(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<LdtkProject>>,
    mut projects: Query<(&Handle<LdtkProject>, &mut LdtkProjectCfg)>,
    entities: Query<(Entity, &LdtkRenderedProject)>,
) {
    for event in events.iter() {
        if let AssetEvent::Modified {
            handle: modified_handle,
        } = event
        {
            for (project_handle, mut project_cfg) in projects.iter_mut() {
                if project_handle == modified_handle {
                    project_cfg.rendered = None;
                }
            }

            for (entity, project) in entities.iter() {
                if modified_handle == &project.0 {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}

pub fn ldtk_entity_cleanup(
    mut commands: Commands,
    level: Query<(&Handle<LdtkProject>, &LdtkProjectCfg), Changed<LdtkProjectCfg>>,
    entities: Query<(Entity, &LdtkRenderedProject)>,
) {
    for (project_handle, project_cfg) in level.iter() {
        match project_cfg.rendered {
            None => {}
            Some(rendered) => match project_cfg.render_type {
                LdtkRenderType::SingleLevel(current) if rendered != current => {}
                _ => continue,
            },
        }

        for (entity, project) in entities.iter() {
            if project_handle == &project.0 {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
#[derive(Debug)]
pub struct EntitySpawn {
    pub project: Handle<LdtkProject>,
    pub translation: Vec3,
    pub size: Vec2,
    pub entity: EntityInstance,
    pub parent_id: u32,
}

pub fn render_ldtk_projects(
    mut commands: Commands,
    mut projects: Query<(&Handle<LdtkProject>, &mut LdtkProjectCfg, &Transform)>,
    mut entity_spawner: EventWriter<EntitySpawn>,
    project_assets: Res<Assets<LdtkProject>>,
) {
    for (project_handle, mut project_cfg, transform) in projects.iter_mut() {
        match project_cfg.render_type {
            LdtkRenderType::SingleLevel(current) => {
                match project_cfg.rendered {
                    Some(l) if l == current => continue,
                    _ => {}
                }

                let project = match project_assets.get(project_handle.clone()) {
                    Some(p) => p,
                    None => continue,
                };

                render_single_ldtk_level(
                    &mut commands,
                    transform,
                    current,
                    false,
                    project_handle,
                    project,
                    &mut entity_spawner,
                    &project_cfg.attach_enums,
                );

                project_cfg.rendered = Some(current);
            }
            LdtkRenderType::FullWorld => {
                if project_cfg.rendered.is_some() {
                    continue;
                }

                let project = match project_assets.get(project_handle.clone()) {
                    Some(p) => p,
                    None => continue,
                };

                for level in 0..project.project.levels.len() {
                    render_single_ldtk_level(
                        &mut commands,
                        transform,
                        level,
                        true,
                        project_handle,
                        project,
                        &mut entity_spawner,
                        &project_cfg.attach_enums,
                    )
                }

                project_cfg.rendered = Some(1);
            }
        }
    }
}

fn render_single_ldtk_level(
    commands: &mut Commands,
    transform: &Transform,
    level: usize,
    reposition_level: bool,
    project_handle: &Handle<LdtkProject>,
    project: &LdtkProject,
    entity_spawner: &mut EventWriter<EntitySpawn>,
    attach_enums: &Option<AttachEnumsFunction>,
) {
    debug!(
        "Beginning render pass for project at level {} ({})",
        level, project.project.levels[level].identifier
    );

    commands.insert_resource(ClearColor(
        Color::hex(&project.project.levels[level].bg_color[1..]).unwrap(),
    ));

    commands
        .spawn()
        .insert_bundle(LdtkRenderBundle {
            project: LdtkRenderedProject(project_handle.clone()),
            transform: *transform,

            ..Default::default()
        })
        .with_children(|builder| {
            let level_info = if reposition_level {
                LevelInfo {
                    world_x: project.project.levels[level].world_x as i32,
                    world_y: project.project.levels[level].world_y as i32,
                }
            } else {
                LevelInfo { world_x: 0, world_y: 0 }
            };

            if let Some(layer_instances) = project.project.levels[level].layer_instances.as_ref() {
                for (idx, layer) in layer_instances.iter().enumerate().rev() {
                    debug!("\tBeginning render pass for layer {}", layer.identifier);
                    let tileset_uid = layer.tileset_def_uid.unwrap_or(-1);
                    let layer_uid = layer.layer_def_uid;

                    let layer_info = LayerInfo {
                        grid_width: layer.c_wid as i32,
                        _grid_height: layer.c_hei as i32,
                        grid_cell_size: layer.grid_size as i32,
                        z_index: 50 - idx as i32,
                    };

                    match &layer.layer_instance_type[..] {
                        "Tiles" => {
                            debug!("\t\t{} Generating Tile Layer", idx);
                            for tile in layer.grid_tiles.iter() {
                                display_tile(
                                    &level_info,
                                    layer_info,
                                    tile,
                                    builder,
                                    project,
                                    tileset_uid,
                                    attach_enums,
                                );
                            }
                        }
                        "AutoLayer" => {
                            debug!("\t\t{} Generating AutoTile Layer", idx);
                            for tile in layer.auto_layer_tiles.iter() {
                                display_tile(
                                    &level_info,
                                    layer_info,
                                    tile,
                                    builder,
                                    project,
                                    tileset_uid,
                                    attach_enums,
                                );
                            }
                        }
                        "IntGrid" => match layer.tileset_def_uid {
                            Some(i) => {
                                debug!("\t\t{} Generating IntGrid Layer w/ Tiles", idx);
                                for tile in layer.auto_layer_tiles.iter() {
                                    display_tile(&level_info, layer_info, tile, builder, project, i, attach_enums);
                                }
                            }
                            None => {
                                debug!("\t\t{} Generating IntGrid layer w/ Color Materials", idx);
                                for tile in layer.int_grid_csv.iter() {
                                    display_color(
                                        &level_info,
                                        layer_info,
                                        tile,
                                        builder,
                                        project.int_grid_colors[&layer_uid][*tile as usize].clone(),
                                    )
                                }
                            }
                        },
                        "Entities" => {
                            debug!("\t\t{} Generating Entities Layer", idx);
                            for entity in &layer.entity_instances {
                                let mut extra_ent_defs = ExtraEntDefs::default();
                                for ent in &project.project.defs.entities {
                                    if ent.uid == entity.def_uid {
                                        extra_ent_defs.__tile_id = 0;
                                        extra_ent_defs.__width = ent.width as i32;
                                        extra_ent_defs.__height = ent.height as i32;
                                    }
                                    if let ldtk::RenderMode::Tile = ent.render_mode {
                                        extra_ent_defs.__tile_id = ent.tile_id.unwrap() as i32;
                                        for ts in &project.project.defs.tilesets {
                                            if ts.uid == ent.tileset_id.unwrap() {
                                                extra_ent_defs.__scale = ent.width as f32 / ts.tile_grid_size as f32;
                                            }
                                        }
                                    }
                                }

                                display_entity(
                                    &level_info,
                                    layer_info,
                                    entity,
                                    builder,
                                    &extra_ent_defs,
                                    entity_spawner,
                                    project_handle.clone(),
                                );
                            }
                        }
                        _ => {
                            error!("\t\t{} Not implemented", idx);
                        }
                    }
                }
            }
        });
}

fn display_tile(
    level_info: &LevelInfo,
    layer_info: LayerInfo,
    tile: &TileInstance,
    builder: &mut ChildBuilder,
    project: &LdtkProject,
    tileset_uid: i64,
    attach_enums: &Option<AttachEnumsFunction>,
) {
    let flip_x = (tile.f & 0b01) != 0;
    let flip_y = (tile.f & 0b10) != 0;

    let handle = project.spritesheets[&tileset_uid].clone();

    let mut commands = builder.spawn();
    commands.insert_bundle(SpriteSheetBundle {
        transform: Transform {
            translation: convert_to_world(
                layer_info.grid_cell_size,
                layer_info.grid_cell_size,
                tile.px[0] as i32 + level_info.world_x,
                tile.px[1] as i32 + level_info.world_y,
                layer_info.z_index,
            ),
            ..Default::default()
        },
        sprite: TextureAtlasSprite {
            index: tile.t as u32,
            flip_x,
            flip_y,
            ..Default::default()
        },
        texture_atlas: handle,
        ..Default::default()
    });

    if let Some(f) = attach_enums {
        f(
            &mut commands,
            AttachEnumsData {
                project,
                tileset_uid,
                tile,
            },
        );
    }
}

fn display_entity(
    level_info: &LevelInfo,
    layer_info: LayerInfo,
    entity: &EntityInstance,
    builder: &mut ChildBuilder,
    extra_ent_defs: &ExtraEntDefs,
    entity_spawner: &mut EventWriter<EntitySpawn>,
    project_handle: Handle<LdtkProject>,
) {
    entity_spawner.send(EntitySpawn {
        translation: convert_to_world(
            extra_ent_defs.__width,
            extra_ent_defs.__height,
            entity.grid[0] as i32 * layer_info.grid_cell_size + level_info.world_x,
            entity.grid[1] as i32 * layer_info.grid_cell_size + level_info.world_y,
            layer_info.z_index,
        ),
        size: Vec2::new(extra_ent_defs.__width as f32, extra_ent_defs.__height as f32),
        entity: (*entity).clone(),
        project: project_handle,
        parent_id: builder.parent_entity().id(),
    })
}

fn display_color(
    level_info: &LevelInfo,
    layer_info: LayerInfo,
    tile: &i64,
    builder: &mut ChildBuilder,
    handle: Handle<ColorMaterial>,
) {
    let x = *tile as i32 % layer_info.grid_width;
    let y = *tile as i32 / layer_info.grid_width;
    builder.spawn().insert_bundle(SpriteBundle {
        material: handle,
        sprite: Sprite::new(Vec2::new(
            layer_info.grid_cell_size as f32,
            layer_info.grid_cell_size as f32,
        )),
        transform: Transform {
            translation: convert_to_world(
                layer_info.grid_cell_size,
                layer_info.grid_cell_size,
                x * layer_info.grid_cell_size + level_info.world_x,
                y * layer_info.grid_cell_size + level_info.world_y,
                layer_info.z_index,
            ),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn convert_to_world(width: i32, height: i32, x: i32, y: i32, z: i32) -> Vec3 {
    let world_x = (x as f32) - (width as f32 / 2.);
    let world_y = (y as f32) + (height as f32 / 2.);
    let world_z = z as f32;
    Vec3::new(world_x, -world_y, world_z)
}
