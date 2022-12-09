use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::{
    constants::{PLAYER_IMPULSE_MAGNITUDE, PLAYER_SIZE},
    input::ShootEvent,
};

// COMPONENTS

#[derive(Component)]
pub struct CannonBall {}

// SYSTEMS

// Spawns and shoots a cannon ball when a ShootEvent is triggered
pub fn shoot_cannon_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_shoot: EventReader<ShootEvent>,
) {
    for ev in ev_shoot.iter() {
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: PLAYER_SIZE / 2.0,
                    subdivisions: 16,
                })),
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(0.3, 0.3, 0.3),
                    perceptual_roughness: 0.3,
                    metallic: 0.8,
                    ..default()
                }),
                transform: Transform::from_translation(ev.position),
                ..default()
            })
            .insert(CannonBall {})
            .insert(Collider::ball(PLAYER_SIZE))
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(RigidBody::Dynamic)
            .insert(Damping {
                linear_damping: 0.1,
                angular_damping: 0.2,
            })
            .insert(ColliderMassProperties::Density(1.0))
            .insert(GravityScale(0.0))
            .insert(Friction {
                coefficient: 2.0,
                combine_rule: CoefficientCombineRule::Max,
            })
            .insert(Restitution {
                coefficient: 0.0,
                combine_rule: CoefficientCombineRule::Max,
            })
            .insert(ExternalForce {
                force: Vec3::new(0.0, 0.0, 0.0),
                torque: Vec3::new(0.0, 0.0, 0.0),
            })
            .insert(ExternalImpulse {
                impulse: ev.direction * PLAYER_IMPULSE_MAGNITUDE,
                torque_impulse: Vec3::new(0.0, 0.0, 0.0),
            });
    }
}
