mod schema;

pub use schema::*;
use std::{
    fs::File,
    path::{Path, PathBuf},
};

impl Project {
    pub fn new<P: AsRef<Path>>(f: P) -> Self {
        let mut o = Self::load_project(&f);
        if o.external_levels {
            o.load_external_levels(f);
        }
        o
    }

    pub fn load_project<P: AsRef<Path>>(f: P) -> Self {
        let file = File::open(f).expect("project file not found");
        let o: Self = serde_json::from_reader(file).expect("error while reading");
        o
    }

    pub fn clear_levels(&mut self) {
        self.levels = Vec::new();
    }

    pub fn load_external_levels<P: AsRef<Path>>(&mut self, f: P) {
        if self.external_levels {
            let mut all_level_files: Vec<PathBuf> = Vec::new();
            for level in &mut self.levels {
                let level_file_path = level.external_rel_path.as_ref().expect("missing_level");
                all_level_files.push(level_file_path.into());
            }

            self.clear_levels();

            for file in &all_level_files {
                let mut full_path = PathBuf::new();
                let parent = f.as_ref().parent().unwrap().to_str().unwrap();
                full_path.push(parent);
                full_path.push("/");
                full_path.push(&file);
                println!("opening {:#?}", full_path);
                let level_ldtk = Level::new(full_path);
                self.levels.push(level_ldtk);
            }
        }
    }

    pub fn get_level(&self, uid: i64) -> Option<&Level> {
        for level in &self.levels {
            if level.uid == uid {
                return Some(level);
            }
        }
        None
    }
}

impl Level {
    pub fn new<P: AsRef<Path>>(f: P) -> Self {
        let file = File::open(f).expect("level file not found");
        let o: Level = serde_json::from_reader(file).expect("error while reading");
        o
    }
}

impl Clone for EntityInstanceTile {
    fn clone(&self) -> Self {
        Self {
            src_rect: self.src_rect.clone(),
            tileset_uid: self.tileset_uid,
        }
    }
}

impl Clone for EntityInstance {
    fn clone(&self) -> Self {
        Self {
            grid: self.grid.clone(),
            identifier: self.identifier.clone(),
            pivot: self.pivot.clone(),
            tile: self.tile.clone(),
            def_uid: self.def_uid,
            field_instances: self.field_instances.clone(),
            height: self.height,
            px: self.px.clone(),
            width: self.width,
        }
    }
}

impl Clone for FieldInstance {
    fn clone(&self) -> Self {
        Self {
            identifier: self.identifier.clone(),
            field_instance_type: self.field_instance_type.clone(),
            value: self.value.clone(),
            def_uid: self.def_uid,
            real_editor_values: self.real_editor_values.clone(),
        }
    }
}

impl Clone for TileInstance {
    fn clone(&self) -> Self {
        Self {
            d: self.d.clone(),
            f: self.f.clone(),
            px: self.px.clone(),
            src: self.src.clone(),
            t: self.t.clone(),
        }
    }
}
