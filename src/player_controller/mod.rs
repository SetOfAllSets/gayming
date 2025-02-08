use std::any::TypeId;

use avian3d::prelude::*;
use bevy::{ecs::system::SystemState, prelude::*};
pub use components::Player;
use components::*;
mod camera;
mod components;

pub struct PlayerControllerPlugin;
impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .register_type::<MovingPlatform>()
            .add_systems(Startup, setup)
            .add_systems(Update, (camera::move_camera, camera::rotate_player))
            .add_systems(FixedUpdate, move_player)
            .add_systems(
                PhysicsSchedule,
                (
                    move_with_ground.ambiguous_with_all(),
                    move_platform.ambiguous_with_all(),
                )
                    .after(PhysicsStepSet::Last)
                    .ambiguous_with_all(),
            );
    }
}

fn setup(world: &mut World) {
    world
        .register_component_hooks::<Player>()
        .on_add(|mut world, entity, _| {
            let player = world.get::<Player>(entity).unwrap().clone();
            world.commands().entity(entity).insert({
                (
                    LockedAxes::new().lock_rotation_x().lock_rotation_z(),
                    Collider::capsule(player.collider_radius, player.collider_height),
                    RigidBody::Dynamic,
                    ExternalForce::default().with_persistence(false),
                    PlayerData::default(),
                )
            });
            let camera = Camera3d::default();
            let projection = Projection::from(PerspectiveProjection {
                fov: 90f32.to_radians(),
                ..default()
            });
            let camera_parent = world.commands().spawn(PlayerCameraChild::default()).id();
            world
                .commands()
                .entity(entity)
                .add_children(&[camera_parent]);
            world
                .commands()
                .entity(camera_parent)
                .insert((camera, projection));

            let parent_ray_caster = RayCaster::new(
                Vec3::ZERO.with_y(0.0 - player.collider_height / 2.0 - player.collider_radius),
                Dir3::NEG_Y,
            )
            .with_max_distance(0.0)
            .with_query_filter(SpatialQueryFilter::from_excluded_entities([entity]));
            let floor_caster_parent = world
                .commands()
                .spawn(PlayerFloorAttatchmentChild::default())
                .id();
            world
                .commands()
                .entity(entity)
                .add_children(&[floor_caster_parent]);
            world
                .commands()
                .entity(floor_caster_parent)
                .insert(parent_ray_caster);
        });
}

fn move_player(
    mut query: Query<(&Player, &mut ExternalForce, &Transform, &mut PlayerData)>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    for (player, mut external_force, transform, mut player_data) in &mut query {
        let mut movement = Vec3::default();
        if keyboard.pressed(KeyCode::KeyW) {
            movement += transform.forward().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyQ) {
            movement += transform.left().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            movement += transform.back().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            movement += transform.right().as_vec3();
        }
        movement = movement.normalize_or_zero() * player.grounded_speed;
        player_data.movement_velocity = movement;
    }
}

fn move_with_ground(
    world: &mut World,
    system_state: &mut SystemState<(
        Query<&RayHits, With<PlayerFloorAttatchmentChild>>,
        Query<(&mut LinearVelocity, &mut AngularVelocity), With<Player>>,
        Query<&mut PlayerData>,
        Res<Time>,
    )>,
) {
    let mut floor_linear_velocity = Vec3::ZERO;
    let mut floor_angular_velocity = Vec3::ZERO;
    let mut hit_entity = Option::<Entity>::None;
    {
        let (ray_query, mut player_query, mut player_data_query, time) =
            system_state.get_mut(world);
        for hits in &ray_query {
            for hit in hits.iter() {
                hit_entity = Some(hit.entity);
            }
        }
    }
    if hit_entity.is_some() {
        let components = world.inspect_entity(hit_entity.unwrap());
        for component in components {
            if component.type_id() == Some(TypeId::of::<LinearVelocity>()) {
                floor_linear_velocity = world.get::<LinearVelocity>(hit_entity.unwrap()).unwrap().0;
            }
            if component.type_id() == Some(TypeId::of::<AngularVelocity>()) {
                floor_angular_velocity =
                    world.get::<AngularVelocity>(hit_entity.unwrap()).unwrap().0;
            }
        }
    }
    let mut player_entity = Option::<Entity>::None;
    {
        let (mut ray_query, mut player_query, mut player_data_query, time) =
            system_state.get_mut(world);
        for (mut linear_velocity, mut angular_velocity) in &mut player_query {
            for mut player_data in &mut player_data_query {
                if floor_linear_velocity != Vec3::default() {
                    linear_velocity.0 = floor_linear_velocity;
                    linear_velocity.0 += player_data.movement_velocity;
                }
                if floor_angular_velocity != Vec3::default() {
                    angular_velocity.0 = floor_angular_velocity;
                }
                println!("{:#?}", floor_linear_velocity);
                //external_force
                //    .apply_impulse(floor_linear_velocity * mass.value() * time.delta_secs());
            }
        }
    }
    if hit_entity.is_some() && player_entity.is_some() {
        world
            .commands()
            .entity(hit_entity.unwrap())
            .add_child(player_entity.unwrap());
    } else if player_entity.is_some() {
        let parent = world.get::<Parent>(player_entity.unwrap());
        if parent.is_some() {
            let parent = parent.unwrap().get();
            world
                .commands()
                .entity(parent)
                .remove_children(&[player_entity.unwrap()]);
        }
    }
}

fn move_platform(mut query: Query<&mut LinearVelocity, With<MovingPlatform>>) {
    for mut velocity in &mut query {
        velocity.0.z = 10.0;
    }
}

/*
Make a moving platform to test that it actually moves w/ it :3
*/
