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
* Make AssetLoader return the "default asset type"
* "dependent paths": ensure dependent non-asset files get copied to .import when the asset isnt redirected
* dont reload assets if they are already loaded. reloads should only happen on metadata change
* "derived assets": assets generated from other assets. this is generally an optimized/precooked version of the asset
    * Editor: Source("assets") + Destination(".imported")
    * Game: Source(1-".imported" else 2-"assets")
    * write derived asset path to source meta
    * import(path)
        1. load(path)
        2. derive()
            * if derivation registered, run and save
                * if derives arent registered for all assets, fail
            * else copy source + meta 
        3. save(path)
* track which assets are still alive in SourceInfo
* imported artifact cleanup
* store import hash in meta
* reconcile serializer vs importer terminology
* Hot reload meta changes
* UUIDs for asset loaders
* Importer versions


## 

Server
  Loader:
    * importers: AssetSerializerTypeUuid
  * register_serializer()
    * HashMap{Uuid, Box{dyn AssetSerializer}}
  * save(path: P, asset: T) { get_serializer(T::UUID).serialize(asset) destination_io.save(path, bytes)}
  