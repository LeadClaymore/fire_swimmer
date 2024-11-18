use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use serde::Deserialize;

use crate::scorch::Scorch;

pub struct EnemyPlugin;

const ENEMY_FORCE_STRENGTH: f32 = 999.0;

#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy_info: EnemyInfo,
}

#[allow(dead_code)]
impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, enemy_movement_system)
            //.add_systems(Startup, setup_test_enemy)
        ;
    }
}

/// Information on enemies
#[derive(Component, Debug, Clone, Copy, Deserialize)]
#[allow(dead_code)]
pub struct EnemyInfo {
    pub e_type: EnemyType,
    pub health: f32,
    pub move_speed: f32,
    pub dmg: f32,
    pub range: f32,
    pub size: f32,
    pub cooldown: f32,
    pub active_cooldown: f32,
}

impl EnemyInfo {
    /// this reduces health by the damage taken and returns if it is at or bellow 0
    pub fn take_dmg(&mut self, dmg: f32) -> bool {
        self.health -= dmg;
        if self.health <= 0.0 {
            return true;
        }
        println!("dmg! ");
        return false;
    }

    /// this causes the death effect, currently does not do anything
    pub fn death_effect(&mut self) {
        //TODO Nothing for now
    }

    pub fn contact_dmg(&self) -> f32 {
        self.dmg
    }

    pub fn speed(&self) -> f32 {
        self.move_speed
    }

    pub fn is_within_range(&self, dist: f32) -> bool {
        dist < self.range
    }

    pub fn handle_shooting(&mut self, curr_time: f32) -> bool {
        if curr_time > self.active_cooldown + self.cooldown {
            self.active_cooldown = curr_time;
            return true;
        }
        return false;
    }
}

/// the type of enemy
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[allow(dead_code)]
pub enum EnemyType {
    RunDown,
    Ranged,
    Stationary,
}

/// enum to store all the projectile structs
#[derive(Component, Debug, Clone, Copy, Deserialize, PartialEq)]
#[allow(dead_code)]
pub enum ProjectileType {
    Contact(ContactProj),
}

#[allow(dead_code)]
impl ProjectileType {
    pub fn default() -> ProjectileType {
        ProjectileType::Contact(ContactProj {
            dmg: 10.0,
            spd: 10.0,
            size: 10.0,
        })
    }



    pub fn get_size(&self) -> f32 {
        match self {
            ProjectileType::Contact(foo) => foo.size,
            //_ => 0.0,
        }
    }

    pub fn get_dmg(&self) -> f32 {
        match self {
            ProjectileType::Contact(foo) => foo.dmg,
            //_ => 0.0,
        }
    }

    pub fn get_spd(&self) -> f32 {
        match self {
            ProjectileType::Contact(foo) => foo.spd,
            //_ => 0.0,
        }
    }
}

/// this type of projectile 
/// moves a specific speed, (spd) at size (size) and on contact does (dmg) damage
#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct ContactProj {
    dmg: f32,
    spd: f32,
    size: f32,
}

// TIL: crashed when used mutable transform remember this
/// every update, this hanndles moving enemies
fn enemy_movement_system(
    time: Res<Time>,
    mut commands: Commands,
    mut enemy_query: Query<(&mut Velocity, &mut ExternalImpulse, &Transform, &mut EnemyInfo)>,
    scorch_query: Query<&Transform, With<Scorch>>,
) {
    // takes the transform from scorch, and maps the 2d + z space into a 2d space
    if let Ok(scorch_pos) = scorch_query.get_single()
        .and_then(|scor_tran| Ok(scor_tran.translation.truncate())) 
    {
        // for each enemy in the map
        //TODO need to add culling distance
        for (
            mut e_vel, 
            mut e_imp, 
            e_trans, 
            mut e_info
        ) in enemy_query.iter_mut() {
            // the direction from the enemy to scorch
            let dir = (scorch_pos - e_trans.translation.truncate()).normalize();

            // different enemy types have diffrent movement
            if e_info.e_type == EnemyType::RunDown {
                // apply impulse towards scorch times the force str times the speed of an enemy
                e_imp.impulse += dir * ENEMY_FORCE_STRENGTH * e_info.speed();

            } else if e_info.e_type == EnemyType::Ranged {
                // if scorch is within range, stop moving
                if e_info.is_within_range(scorch_pos.distance(e_trans.translation.truncate())) {
                    e_vel.linvel = Vec2::ZERO;
                    if e_info.handle_shooting(time.elapsed_seconds()) {
                        ranged_enemy_shoot( 
                            &mut commands, 
                            //TODO I think I need a ofset for spawning
                            e_trans.translation.truncate() + dir * (e_info.size + 20.0),
                            dir,
                            ProjectileType::default(),
                            //*e_info,
                        );
                    }

                // if scorch is outside of range, move to scorch, at the enemies speed * const
                } else {
                    e_imp.impulse += dir * ENEMY_FORCE_STRENGTH * e_info.speed();
                }
            } else if e_info.e_type == EnemyType::Stationary {
                // for now nothing, might add turning later
            }
        }
    }
}

/// spawns an enemy on the location specified, with the info specified,
pub fn spawn_enemy(
    commands: &mut Commands,
    e_pos: Vec2,
    e_info: EnemyInfo,
    e_size: f32,
) {
    commands
        .spawn((
            // position and enemy info
            TransformBundle::from(Transform::from_xyz(e_pos.x, e_pos.y, 0.0)),
            Collider::ball(e_size),
            e_info,

            // default settings
            CollisionGroups::new(
                // G1 is Scorch, G2 is embers, G3 is blocks, G4 is enemies, G5 is enemy_projectiles
                Group::GROUP_4,
                Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_3,
            ),
            RigidBody::Dynamic,
            Restitution::coefficient(0.5),
            ExternalImpulse::default(),
            Velocity::default(),
            GravityScale(0.0),
            ColliderMassProperties::Density(1.0),
            LockedAxes::ROTATION_LOCKED,
            ActiveEvents::COLLISION_EVENTS,
        ));
}

/// spawns an projectile 
pub fn ranged_enemy_shoot(
    commands: &mut Commands,
    p_pos: Vec2,
    p_dir: Vec2,
    p_type: ProjectileType,
    //e_type: EnemyInfo,
) {
    //println!("shoot");
    commands
        .spawn((
            // from data provided
            TransformBundle::from(Transform::from_xyz(p_pos.x, p_pos.y, 0.0)),
            Collider::ball(p_type.get_size()),
            CollisionGroups::new(
                // G1 is Scorch, G2 is embers, G3 is blocks, G4 is enemies, G5 is enemy_projectiles
                Group::GROUP_5,
                //TODO currently I just want the projectiles interacting with blocks and scorch
                Group::GROUP_1 | Group::GROUP_3,
            ),
            ExternalImpulse {
                impulse: p_dir * p_type.get_spd() * ENEMY_FORCE_STRENGTH * 10.0,
                ..default()
            },
            p_type,

            // default data
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            Velocity::default(),
            GravityScale(0.0),
            ActiveEvents::COLLISION_EVENTS,
        ));
}