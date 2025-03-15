use avian3d::prelude::*;
use bevy::{
    ecs::system::SystemState,
    prelude::*,
};
use camera::*;
pub use components::Player;
use components::*;
mod camera;
mod components;

pub struct PlayerControllerPlugin;
impl Plugin for PlayerControllerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Player>()
            .register_type::<MovingPlatform>()
            .register_type::<PlayerData>()
            .add_systems(Startup, setup)
            .add_systems(Update, camera::move_camera)
            .add_systems(
                PhysicsSchedule,
                (
                    move_platform.ambiguous_with_all(),
                    (get_floor_velocity, move_player, rotate_player)
                        .chain()
                        .ambiguous_with_all(),
                )
                    .before(PhysicsStepSet::First)
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

            let floor_ray_caster = RayCaster::new(
                Vec3::ZERO.with_y(0.0 - player.collider_height / 2.0 - player.collider_radius),
                Dir3::NEG_Y,
            )
            .with_max_distance(0.0)
            .with_query_filter(SpatialQueryFilter::from_excluded_entities([entity]));
            let floor_caster_parent = world.commands().spawn(PlayerFloorCaster::default()).id();
            world
                .commands()
                .entity(entity)
                .add_children(&[floor_caster_parent]);
            world
                .commands()
                .entity(floor_caster_parent)
                .insert(floor_ray_caster);
        });
    world.commands().spawn((
        MovingPlatform,
        ExternalForce::default(),
        RigidBody::Kinematic,
        ColliderConstructor::Cuboid {
            x_length: 20.0,
            y_length: 2.0,
            z_length: 20.0,
        },
    ));
}

fn get_floor_velocity(
    world: &mut World,
    player_query: &mut SystemState<Query<&mut PlayerData, With<Player>>>,
    ray_query: &mut SystemState<
        Query<(&mut RayHits, &RayCaster), (With<PlayerFloorCaster>, Without<Player>)>,
    >,
) {
    let mut entity: Option<Entity> = None;
    let mut hit_point: Option<Vec3> = None;
    for (hits, caster) in ray_query.get_mut(world).iter() {
        for hit in hits.iter() {
            entity = Some(hit.entity);
            hit_point = Some(caster.global_origin());
        }
    }
    let mut last_floor_transform: Option<Transform> = None;
    for player_data in player_query.get_mut(world).iter_mut() {
        last_floor_transform = player_data.last_floor_transform;
    }
    let mut linear_velocity: Option<Vec3> = None;
    let mut angular_velocity: Option<Vec3> = None;
    let mut floor_transform: Option<Transform> = None;
    if let Some(entity) = entity {
        match world.entity(entity).get::<Transform>() {
            None => floor_transform = None,
            Some(origin) => floor_transform = Some(*origin),
        }
        let floor_angular_velocity: Option<Vec3> =
            match world.entity(entity).get::<AngularVelocity>() {
                None => None,
                Some(velocity) => Some(velocity.0),
            };
        let floor_linear_velocity: Option<Vec3> = match world.entity(entity).get::<LinearVelocity>()
        {
            None => None,
            Some(velocity) => Some(velocity.0),
        };
        if floor_transform.is_some()
            && last_floor_transform.is_some()
            && floor_linear_velocity.is_some()
            && floor_angular_velocity.is_some()
            && hit_point.is_some()
        {
            let mut point_movement = hit_point.unwrap() - floor_transform.unwrap().translation;
            point_movement = Quat::from_axis_angle(
                floor_angular_velocity.unwrap().normalize_or_zero(),
                floor_angular_velocity.unwrap().length() * world.resource::<Time>().delta_secs(),
            )
            .mul_vec3(point_movement);
            point_movement += floor_transform.unwrap().translation;
            point_movement -= hit_point.unwrap();
            point_movement /= world.resource::<Time>().delta_secs();
            linear_velocity = Some(point_movement + floor_linear_velocity.unwrap());
        }
    }
    if linear_velocity.is_none() {
        linear_velocity = Some(Vec3::ZERO);
    }
    if angular_velocity.is_none() {
        angular_velocity = Some(Vec3::ZERO);
    }
    for mut player_data in player_query.get_mut(world).iter_mut() {
        player_data.floor_linear_velocity = linear_velocity.unwrap();
        player_data.floor_angular_velocity = angular_velocity.unwrap();
        player_data.last_floor_transform = floor_transform;
        player_data.floor_entity = entity;
    }
}

fn move_player(
    mut query: Query<(
        &mut PlayerData,
        &mut LinearVelocity,
        &mut AngularVelocity,
        &Transform,
        &Player,
        Entity,
        &Collider,
    )>,
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    spatial_query: SpatialQuery,
) {
    for (
        mut player_data,
        mut linear_velocity,
        mut angular_velocity,
        transform,
        player,
        entity,
        collider,
    ) in &mut query
    {
        player_data.movement_linear_velocity = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) {
            player_data.movement_linear_velocity += transform.forward().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyQ) {
            player_data.movement_linear_velocity += transform.left().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyS) {
            player_data.movement_linear_velocity += transform.back().as_vec3();
        }
        if keyboard.pressed(KeyCode::KeyD) {
            player_data.movement_linear_velocity += transform.right().as_vec3();
        }
        player_data.movement_linear_velocity =
            player_data.movement_linear_velocity.normalize_or_zero() * player.grounded_speed;
        let mut simulated_velocity_impact =
            **linear_velocity - player_data.last_unsimulated_velocity;
        let unsimulated_velocity =
            player_data.floor_linear_velocity + player_data.movement_linear_velocity;
        let excluded_entities = Vec::from([entity]);
        simulated_velocity_impact *= 1.0 / (1.0 + time.delta_secs() * 0.8);
        *linear_velocity = LinearVelocity::from(unsimulated_velocity + simulated_velocity_impact);
        *angular_velocity = AngularVelocity::from(player_data.floor_angular_velocity);
        if Dir3::new(unsimulated_velocity.normalize()).is_ok() {
            let hits = spatial_query.shape_hits(
                &Collider::capsule(player.collider_radius, player.collider_height - 0.5),
                Vec3::ZERO.with_y(0.5),
                transform.rotation,
                Dir3::new(unsimulated_velocity.normalize()).unwrap(),
                1,
                &ShapeCastConfig::default().with_max_distance(unsimulated_velocity.length()),
                &SpatialQueryFilter::default().with_excluded_entities(excluded_entities),
            );
            for hit in hits {
                if hit.distance != 0.0 {
                    //unsimulated_velocity = unsimulated_velocity.normalize() * hit.distance;
                }
                println!("ee: {:#?}", time.elapsed());
            }
        }
        player_data.last_unsimulated_velocity = unsimulated_velocity;
    }
}

fn move_platform(
    mut query: Query<(&mut LinearVelocity, &mut AngularVelocity), With<MovingPlatform>>,
) {
    for (mut linear_velocity, mut angular_velocity) in &mut query {
        linear_velocity.0.z = 10.0;
        angular_velocity.0.y = 1.0;
    }
}
