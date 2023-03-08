// Asteroids fall from the sky and roll around the planet.
// If they hit the player, the game ends. If they get hit by a cannon ball or another asteroid, they despawn after exploding.

use bevy::{
    prelude::{
        default, shape, AssetServer, Assets, Commands, Component, Mesh, PbrBundle, Res, ResMut,
        Resource, StandardMaterial, Transform, Vec3,
    },
    time::{Time, Timer, TimerMode},
};
use bevy_rapier3d::prelude::{
    ActiveEvents, CoefficientCombineRule, Collider, ColliderMassProperties, ExternalForce,
    ExternalImpulse, Friction, GravityScale, Restitution, RigidBody,
};
use rand::{thread_rng, Rng};

use crate::common::PLANET_SIZE;

// CONSTANTS

const ASTEROID_SIZE: f32 = 1.0;
const ASTEROID_IMPULSE_MAGNITUDE: f32 = 50.0;
const ASTEROID_SPAWN_DELAY: f32 = 5.0;
const ASTEROID_SPAWN_ALTITUDE: f32 = PLANET_SIZE * 2.0;

// COMPONENTS

#[derive(Component)]
pub struct Asteroid {}

// RESOURCES

#[derive(Resource)]
pub struct AsteroidSpawnTimer(pub Timer);

// STARTUP SYSTEMS

pub fn setup_asteroids(mut commands: Commands) {
    // Insert resouce to keep track of time until the next asteroid is spawned
    commands.insert_resource(AsteroidSpawnTimer(Timer::from_seconds(
        ASTEROID_SPAWN_DELAY,
        TimerMode::Repeating,
    )));
}

// SYSTEMS

pub fn spawn_asteroids(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_timer: ResMut<AsteroidSpawnTimer>,
    time: Res<Time>,
) {
    if spawn_timer.0.tick(time.delta()).finished() {
        // Generate random number generator
        let mut rng = thread_rng();

        // Spawn asteroid at random position above the planet
        let x = rng.gen_range(-1.0..1.0) * 10.0;
        let y = 10.0;
        let z = rng.gen_range(-1.0..1.0) * 10.0;
        let position = Vec3::new(x, y, z).normalize() * ASTEROID_SPAWN_ALTITUDE;

        // Apply impulse to asteroid in random direction towards the planet but not directly at it
        let x = rng.gen_range(-1.0..1.0) * 10.0;
        let y = rng.gen_range(-1.0..1.0) * 10.0;
        let z = rng.gen_range(-1.0..1.0) * 10.0;
        let to_planet = position.normalize();
        let direction = (Vec3::new(x, y, z).cross(to_planet)).normalize();

        commands
            .spawn(PbrBundle {
                mesh: meshes.add(
                    shape::Icosphere {
                        radius: ASTEROID_SIZE / 2.0,
                        subdivisions: 16,
                    }
                    .try_into()
                    .unwrap(),
                ),
                material: materials.add(StandardMaterial {
                    base_color_texture: asset_server
                        .load("textures/asteroid/asteroid_base.png")
                        .into(),
                    normal_map_texture: asset_server
                        .load("textures/asteroid/asteroid_normal.png")
                        .into(),
                    perceptual_roughness: 1.0,
                    metallic: 1.0,
                    ..default()
                }),
                transform: Transform::from_translation(position),
                ..default()
            })
            .insert(Asteroid {})
            .insert(Collider::ball(ASTEROID_SIZE / 2.0))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(RigidBody::Dynamic)
            .insert(ColliderMassProperties::Density(1.0))
            .insert(GravityScale(0.0))
            .insert(Friction {
                coefficient: 1.0,
                combine_rule: CoefficientCombineRule::Min,
            })
            .insert(Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Max,
            })
            .insert(ExternalForce {
                force: Vec3::ZERO,
                torque: Vec3::ZERO,
            })
            .insert(ExternalImpulse {
                impulse: direction * ASTEROID_IMPULSE_MAGNITUDE,
                torque_impulse: Vec3::ZERO,
            });
    }
}
