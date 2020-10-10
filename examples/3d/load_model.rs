use bevy::prelude::*;
use bevy_type_registry::TypeRegistry;

fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_startup_system(setup.system())
        // .add_system(print_world_system.thread_local_system())
        .run();
}

#[allow(unused)]
fn print_world_system(world: &mut World, resources: &mut Resources) {
    let registry = resources.get::<TypeRegistry>().unwrap();
    let dc = DynamicScene::from_world(world, &registry.component.read());
    println!("WORLD");
    println!("{}", dc.serialize_ron(&registry.property.read()).unwrap());
    println!();
    println!();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // let handle: Handle<Mesh> = asset_server.load("models/scene/scene.gltf#Mesh0/Primitive0").unwrap();
    let scene_handle: Handle<Scene> = asset_server.load("models/scene/scene_dependency.gltf");

    // add entities to the world
    commands
        .spawn_scene(scene_handle)
        // .spawn(PbrComponents {
        //     mesh: asset_server
        //         .get_handle("models/scene/scene.gltf#Mesh0/Primitive0")
        //         .unwrap(),
        //     ..Default::default()
        // })
        .spawn(LightComponents {
            transform: Transform::from_translation(Vec3::new(4.0, 5.0, 4.0)),
            ..Default::default()
        })
        // camera
        .spawn(Camera3dComponents {
            transform: Transform::new(Mat4::face_toward(
                Vec3::new(3.0, 2.0, 5.0),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(0.0, 1.0, 0.0),
            )),
            ..Default::default()
        });
}
