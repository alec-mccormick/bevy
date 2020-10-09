use anyhow::Result;
use bevy_asset::{AssetLoader, AssetSerializer, LoadContext};
use bevy_type_registry::TypeUuid;

use super::Mesh;

#[derive(TypeUuid)]
#[uuid = "a8d20e9c-a8b0-4d1b-9899-f40ad05ff5d5"]
pub struct BinaryMeshLoader;

const BINARY_MESH_EXTENSION: &str = "mesh";

impl AssetLoader for BinaryMeshLoader {
    fn load(&self, bytes: Vec<u8>, load_context: &mut LoadContext) -> Result<()> {
        Ok(())
    }

    fn extensions(&self) -> &[&str] {
        &[BINARY_MESH_EXTENSION]
    }
}

#[derive(TypeUuid)]
#[uuid = "a0294291-14d8-4663-a1d6-59067aecfb4d"]
pub struct BinaryMeshSerializer;

impl AssetSerializer for BinaryMeshSerializer {
    type Asset = Mesh;

    fn serialize(&self, asset: &Self::Asset) -> Result<Vec<u8>, anyhow::Error> {
        todo!()
    }

    fn extension(&self) -> &str {
        BINARY_MESH_EXTENSION
    }
}
