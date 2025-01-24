use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
#[require(
    PlayerData,
    RigidBody,
    Collider,
    ShapeCaster,
    LockedAxes,
    ExternalForce,
    GravityScale
)]
pub struct Player {
    pub collider_height: f32,
    pub collider_radius: f32,
    pub max_slope_degrees: f32,
    pub float_height: f32,
    pub grounded_speed: f32,
    pub airborne_sped: f32,
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
            float_height: 2.0,
            grounded_speed: 15.0,
            airborne_sped: 7.5,
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
    pub prev_pushed_down: bool,
    pub jumped: Timer,
}

impl Default for PlayerData {
    fn default() -> Self {
        PlayerData {
            grounded: GroundedState::Ungrounded(UngroundedReason::Airborne),
            ground_distance: None,
            ground_height: None,
            ground_normal: None,
            prev_pushed_down: false,
            jumped: {
                let mut timer = Timer::new(Duration::from_secs_f32(0.05), TimerMode::Once);
                timer.tick(Duration::from_secs(5));
                timer
            },
        }
    }
}

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerShapeCasterChild;

impl Default for PlayerShapeCasterChild {
    fn default() -> Self {
        PlayerShapeCasterChild
    }
}

#[derive(Component, Reflect)]
#[reflect(Component, Default)]
pub struct PlayerCameraChild {
    pub pitch: f32,
}

impl Default for PlayerCameraChild {
    fn default() -> Self {
        PlayerCameraChild {
            pitch: 90f32.to_radians(),
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
