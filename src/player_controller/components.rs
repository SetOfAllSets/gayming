use std::time::Duration;

use bevy::prelude::*;

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct Player {
    pub collider_height: f32,
    pub collider_radius: f32,
    pub max_slope_degrees: f32,
    pub stand_height: f32,
    pub crouch_height: f32,
    pub grounded_speed: f32,
    pub airborne_speed: f32,
    pub horizontal_camera_sensitivity: f32,
    pub vertical_camera_sensitivity: f32,
    pub fov: f32,
    pub ground_friction: f32,
    pub jump_height: f32,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            collider_height: 2.0,
            collider_radius: 2.0,
            max_slope_degrees: 0.75,
            stand_height: 2.0,
            crouch_height: 1.0,
            grounded_speed: 15.0,
            airborne_speed: 7.5,
            horizontal_camera_sensitivity: 0.05,
            vertical_camera_sensitivity: 0.01,
            fov: 90.0,
            ground_friction: 10.0,
            jump_height: 5.0,
        }
    }
}

#[derive(Component, Reflect, Debug, PartialEq)]
#[reflect(Component, Default)]
pub struct PlayerData {
    pub grounded: GroundedState,
    pub ground_distance: Option<f32>,
    pub ground_height: Option<f32>,
    pub ground_normal: Option<Dir3>,
    pub jumped: Timer,
    pub crouching: bool,
    pub last_unsimulated_velocity: Vec3,
    pub movement_linear_velocity: Vec3,
    pub last_floor_transform: Option<Transform>,
    pub floor_linear_velocity: Vec3,
    pub floor_angular_velocity: Vec3,
    pub floor_entity: Option<Entity>,
}

impl Default for PlayerData {
    fn default() -> Self {
        PlayerData {
            grounded: GroundedState::Ungrounded(UngroundedReason::Airborne),
            ground_distance: None,
            ground_height: None,
            ground_normal: None,
            jumped: {
                let mut timer = Timer::new(Duration::from_secs_f32(0.5), TimerMode::Once);
                timer.tick(Duration::from_secs(5));
                timer
            },
            crouching: false,
            last_unsimulated_velocity: Vec3::ZERO,
            movement_linear_velocity: Vec3::ZERO,
            last_floor_transform: None,
            floor_linear_velocity: Vec3::ZERO,
            floor_angular_velocity: Vec3::ZERO,
            floor_entity: None,
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerCameraChild {
    pub pitch: f32,
    pub distance: f32,
}

impl Default for PlayerCameraChild {
    fn default() -> Self {
        PlayerCameraChild {
            pitch: 0.0,
            distance: 2.0,
        }
    }
}

#[derive(Reflect, Debug, PartialEq)]
pub enum GroundedState {
    Grounded,
    Ungrounded(UngroundedReason),
}

#[derive(Reflect, Debug, PartialEq)]
pub enum UngroundedReason {
    Airborne,
    SteepSlope,
}

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerFloorCaster;

impl Default for PlayerFloorCaster {
    fn default() -> Self {
        PlayerFloorCaster
    }
}

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct MovingPlatform;

impl Default for MovingPlatform {
    fn default() -> Self {
        MovingPlatform
    }
}
