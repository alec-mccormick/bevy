use crate::{AssetIo, AssetIoError, AssetMeta, AssetServer, Assets, Handle, HandleId, RefChangeChannel, SourceMeta, path::AssetPath};
use anyhow::Result;
use bevy_ecs::{Res, ResMut, Resource};
use bevy_type_registry::TypeUuid;
use bevy_utils::HashMap;
use crossbeam_channel::{Receiver, Sender};
use downcast_rs::{impl_downcast, Downcast};
use std::path::Path;
use uuid::Uuid;

/// A loader for a given asset of type `T`
pub trait AssetLoader: Send + Sync + 'static {
    fn load(&self, bytes: Vec<u8>, load_context: &mut LoadContext) -> Result<(), anyhow::Error>;
    fn extensions(&self) -> &[&str];
}

pub trait Asset: TypeUuid + AssetDynamic {}

pub trait AssetDynamic: Downcast + Send + Sync + 'static {
    fn type_uuid(&self) -> Uuid;
}
impl_downcast!(AssetDynamic);

impl<T> Asset for T where T: TypeUuid + AssetDynamic {}

impl<T> AssetDynamic for T
where
    T: Send + Sync + 'static + TypeUuid,
{
    fn type_uuid(&self) -> Uuid {
        Self::TYPE_UUID
    }
}

pub struct LoadedAsset {
    pub(crate) value: Option<Box<dyn AssetDynamic>>,
    pub(crate) dependencies: Vec<AssetPath<'static>>,
}

impl LoadedAsset {
    pub fn new<T: Asset>(value: T) -> Self {
        Self {
            value: Some(Box::new(value)),
            dependencies: Vec::new(),
        }
    }

    pub fn with_dependency(mut self, asset_path: AssetPath) -> Self {
        self.dependencies.push(asset_path.to_owned());
        self
    }

    pub fn with_dependencies(mut self, asset_paths: Vec<AssetPath<'static>>) -> Self {
        self.dependencies.extend(asset_paths);
        self
    }
}

pub struct LoadContext<'a> {
    pub(crate) ref_change_channel: &'a RefChangeChannel,
    pub(crate) asset_io: &'a dyn AssetIo,
    pub(crate) labeled_assets: HashMap<Option<String>, LoadedAsset>,
    pub(crate) path: &'a Path,
    pub(crate) version: usize,
}

impl<'a> LoadContext<'a> {
    pub(crate) fn new(
        path: &'a Path,
        ref_change_channel: &'a RefChangeChannel,
        asset_io: &'a dyn AssetIo,
        version: usize,
    ) -> Self {
        Self {
            ref_change_channel,
            asset_io,
            labeled_assets: Default::default(),
            version,
            path,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn has_labeled_asset(&self, label: &str) -> bool {
        self.labeled_assets.contains_key(&Some(label.to_string()))
    }

    pub fn set_default_asset(&mut self, asset: LoadedAsset) {
        self.labeled_assets.insert(None, asset);
    }

    pub fn set_labeled_asset(&mut self, label: &str, asset: LoadedAsset) {
        assert!(!label.is_empty());
        self.labeled_assets.insert(Some(label.to_string()), asset);
    }

    pub fn get_handle<I: Into<HandleId>, T: Asset>(&self, id: I) -> Handle<T> {
        Handle::strong(id.into(), self.ref_change_channel.sender.clone())
    }

    pub fn read_asset_bytes<P: AsRef<Path>>(&self, path: P) -> Result<Vec<u8>, AssetIoError> {
        self.asset_io.load_path(path.as_ref())
    }

    pub fn set_meta(&self, meta: &mut SourceMeta) {
        let mut asset_metas = Vec::new();
        for (label, asset) in self.labeled_assets.iter() {
            asset_metas.push(AssetMeta {
                dependencies: asset.dependencies.clone(),
                label: label.clone(),
                type_uuid: asset.value.as_ref().unwrap().type_uuid(),
            });
        }
        meta.assets = asset_metas;
    }
}

/// The result of loading an asset of type `T`
pub struct AssetResult<T: Resource> {
    pub asset: T,
    pub id: HandleId,
    pub version: usize,
}

/// A channel to send and receive [AssetResult]s
pub struct AssetLifecycleChannel<T: Resource> {
    pub sender: Sender<AssetLifecycleEvent<T>>,
    pub receiver: Receiver<AssetLifecycleEvent<T>>,
}

pub enum AssetLifecycleEvent<T: Resource> {
    Create(AssetResult<T>),
    Free(HandleId),
}

pub trait AssetLifecycle: Downcast + Send + Sync + 'static {
    fn create_asset(&self, id: HandleId, asset: Box<dyn AssetDynamic>, version: usize);
    fn free_asset(&self, id: HandleId);
}
impl_downcast!(AssetLifecycle);

impl<T: AssetDynamic> AssetLifecycle for AssetLifecycleChannel<T> {
    fn create_asset(&self, id: HandleId, asset: Box<dyn AssetDynamic>, version: usize) {
        if let Ok(asset) = asset.downcast::<T>() {
            self.sender
                .send(AssetLifecycleEvent::Create(AssetResult {
                    id,
                    asset: *asset,
                    version,
                }))
                .unwrap()
        } else {
            panic!("failed to downcast asset to {}", std::any::type_name::<T>());
        }
    }

    fn free_asset(&self, id: HandleId) {
        self.sender.send(AssetLifecycleEvent::Free(id)).unwrap();
    }
}

impl<T: Resource> Default for AssetLifecycleChannel<T> {
    fn default() -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded();
        AssetLifecycleChannel { sender, receiver }
    }
}

/// Reads [AssetResult]s from an [AssetChannel] and updates the [Assets] collection and [LoadState] accordingly
pub fn update_asset_storage_system<T: Asset + AssetDynamic>(
    asset_server: Res<AssetServer>,
    mut assets: ResMut<Assets<T>>,
) {
    asset_server.update_asset_storage(&mut assets);
}
