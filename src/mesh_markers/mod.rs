use bevy::{asset::AssetIndex, gltf::GltfMesh, prelude::*};
use avian3d::prelude::*;

pub struct MeshMarkerPlugin;
impl Plugin for MeshMarkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, instantiate_meshes).register_type::<RampMarker>();
        //app.init_asset::<Mesh>();
    }
}

fn instantiate_meshes(world: &mut World) {
    world.register_component_hooks::<RampMarker>().on_add(|mut world, entity, _| {
        println!("a");
        let asset_server = world.resource::<AssetServer>();
        println!("b");
        let handle = asset_server.load::<GltfMesh>(GltfAssetLabel::Mesh(0).from_asset("blueprints/Ramp.glb"));
        println!("{:#?}", handle.id());
        println!("c");
        let mesh = world.resource::<Assets<GltfMesh>>().get(handle.id());
        println!("{:#?}", mesh);
        let mesh = world.resource::<Assets<Mesh>>().get(AssetIndex::from_bits(mesh.unwrap().index as u64));
        println!("{:#?}", mesh);
        let collider = Collider::trimesh_from_mesh(mesh.unwrap()).unwrap();
        world.commands().entity(entity).insert((collider, RigidBody::Static));
    });
}

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct RampMarker;

impl Default for RampMarker {
    fn default() -> Self {
        RampMarker
    }
}