# Bevy Asset

## Terms

* Asset: A type T that is stored in an `Assets<T>` collection, uniquely identified by an AssetId
* Asset Id: A unique identifier for an asset
* Asset Source: A Vec<u8> loaded from a given "path", uniquely identified by an AssetSourceId. These are generally the "source files" of an asset, such as "image.png", "scene.gltf", "sound.mp3", etc. An asset source can produce zero-to-many Assets. 
* Asset Source id: a unique identifier for an asset source
* Asset Source Metadata: Generated metadata stored alongside an Asset Source that provides information about how the asset was loaded, unique id of the source, and Asset Metadata.
* Asset Metadata: generated metadata for a specific Asset produced by an Asset Source. Contains an AssetId and asset dependencies

## Todo

* Make WeakHandle its own type?
    * implies that all Bundle handles are strong
    * implies Bundles cannot impl Default
    * breaks ergo
* Avoid re-imports when moving assets while still using asset path?
    * maybe we still use asset id
* real untyped asset handle. load_folder returns untyped
* dont reload assets if they are already loaded. reloads should only happen on metadata change
* "derived assets": assets generated from other assets. this is generally an optimized/precooked version of the asset
* Hot reload meta changes
* UUIDs for asset loaders
* Importer versions