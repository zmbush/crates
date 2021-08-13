#!/bin/bash

quicktype https://ldtk.io/files/JSON_SCHEMA.json --src-lang schema -o ldtk/src/schema.rs -t Project --visibility public --derive-debug --density normal
sed -i -e 's/extern crate serde_derive;/use serde::{Deserialize, Serialize};/g' ldtk/src/schema.rs
rustfmt ldtk/src/schema.rs
