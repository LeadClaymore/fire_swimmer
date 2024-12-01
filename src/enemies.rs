use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use serde::Deserialize;

use crate::{asset_loader::SceneAsset, scorch::Scorch, state_system::AppState};

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
            .add_systems(
                Update, 
                (enemy_movement_system).run_if(in_state(AppState::InGame))
            )
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
    pub stunned_until: f32,
    pub within_range: bool,
    pub damage_per_frame: f32,
    pub moveable: bool,
}

// This should not be called instead to fill in gaps in the other enemy spawns
impl Default for EnemyInfo {
    fn default() -> Self {
        Self {
            e_type: EnemyType::RunDown,
            health: 100.0,
            move_speed: 10.0,
            dmg: 10.0,
            range: 250.0,
            size: 25.0,
            cooldown: 1.0,
            active_cooldown: 0.0,
            stunned_until: 0.0,
            within_range: false,
            damage_per_frame: 0.0,
            moveable: true,
        }
    }
}

impl EnemyInfo {
    /// this reduces health by the damage taken and returns if it is at or bellow 0
    pub fn take_dmg(&mut self, dmg: f32) -> bool {
        self.health -= dmg;
        if self.health <= 0.0 {
            return true;
        }
        //println!("dmg! ");
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

    pub fn is_stunned(self, curr_time: f32) -> bool {
        !(curr_time > self.stunned_until)
    }

    pub fn stun_until(&mut self, time_unstunned: f32) {
        self.stunned_until = time_unstunned;
    }

    pub fn set_active(&mut self) {
        println!("enemy in range");
        self.within_range = true;
    }

    pub fn is_active(&self) -> bool {
        self.within_range
    }

    #[allow(dead_code)]
    pub fn get_dpf(&self) -> f32 {
        self.damage_per_frame
    }

    pub fn add_dpf(&mut self, added_dmg: f32) {
        if added_dmg + self.damage_per_frame > 0.0 {
            self.damage_per_frame += added_dmg;
        } else {
            self.clear_dpf();
        }
    }

    pub fn clear_dpf(&mut self) {
        self.damage_per_frame = 0.0;
    }

    pub fn is_moveable(&self) -> bool{
        self.moveable
    }
}

/// the type of enemy
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Default)]
#[allow(dead_code)]
pub enum EnemyType {
    #[default]
    RunDown,
    Ranged,
    Stationary,
    StationaryRanged,
    Summoner,
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
    s_trans_query: Query<&Transform, With<Scorch>>,
    asset_server: Res<SceneAsset>,

    rc: Res<RapierContext>,
    is_s_query: Query<(), With<Scorch>>,
    is_p_query: Query<(), With<ProjectileType>>,
) {
    // takes the transform from scorch, and maps the 2d + z space into a 2d space
    if let Ok(scorch_pos) = s_trans_query.get_single()
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
            // I dislike the inefficentcy in this.
            // Its fine for now but if I want more enemies this will be a problem
            //TODO reduce load by culling calls for inactive enemies
            if !e_info.is_stunned(time.elapsed_seconds()) && e_info.is_active() {
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
                                &asset_server,
                            );
                        }

                    // if scorch is outside of range, move to scorch, at the enemies speed * const
                    } else {
                        e_imp.impulse += dir * ENEMY_FORCE_STRENGTH * e_info.speed();
                    }
                } else if e_info.e_type == EnemyType::Stationary {
                    // for now nothing, might add turning later
                } else if e_info.e_type == EnemyType::StationaryRanged {
                    //println!("LOS");
                    if let Some((rc_entity, _toi)) = &rc.cast_ray(
                        e_trans.translation.truncate() + dir * (e_info.size + 1.0),
                        dir,
                        scorch_pos.distance(e_trans.translation.truncate()),
                        false,
                        QueryFilter::default().exclude_sensors(),
                    ) {
                        //TODO fix multishot from stationary ranged
                        // Need to do a deeper query of objects either learn how to use the QueryFilter
                        // or filter through the objects between e and s
                        if 
                            is_s_query.get(*rc_entity).is_ok() 
                            || is_p_query.get(*rc_entity).is_ok() 
                        {
                            if e_info.handle_shooting(time.elapsed_seconds()) {
                                ranged_enemy_shoot( 
                                    &mut commands, 
                                    //TODO I think I need a ofset for spawning
                                    e_trans.translation.truncate() + dir * (e_info.size + 20.0),
                                    dir,
                                    ProjectileType::default(),
                                    //*e_info,
                                    &asset_server,
                                );
                            }
                        } else {
                            //println!("entity not scorch {:?}, with toi: {}", rc_entity, _toi);
                        }
                    } else {
                        println!("stationary enemy raycast error");
                    }
                } else if e_info.e_type == EnemyType::Summoner {
                    // if scorch is within range, stop moving
                    if e_info.is_within_range(scorch_pos.distance(e_trans.translation.truncate())) {
                        e_vel.linvel = Vec2::ZERO;
                        if e_info.handle_shooting(time.elapsed_seconds()) {
                            let s_enemy_size: f32 = 15.0;
                            spawn_enemy( 
                                &mut commands, 
                                //TODO I think I need a ofset for spawning
                                e_trans.translation.truncate() + dir * (e_info.size + s_enemy_size + 1.0),
                                EnemyInfo {
                                    size: s_enemy_size,
                                    move_speed: 7.5,
                                    health: 50.0,
                                    dmg: 7.5,
                                    ..default()
                                },
                                s_enemy_size,
                                &asset_server,
                            );
                        }

                    // if scorch is outside of range, move to scorch, at the enemies speed * const
                    } else {
                        e_imp.impulse += dir * ENEMY_FORCE_STRENGTH * e_info.speed();
                    }
                }
            }
        }
    }
}

#[allow(dead_code, unreachable_patterns)]
/// spawns an enemy on the location specified, with the info specified,
pub fn spawn_enemy(
    commands: &mut Commands,
    e_pos: Vec2,
    e_info: EnemyInfo,
    e_size: f32,
    asset_server: &Res<SceneAsset>,
) {
    // this sets the texture based on what the enemy is
    let t_texture = match e_info.e_type {
        EnemyType::RunDown => asset_server.t_enemy.clone(),
        EnemyType::Ranged => asset_server.t_enemy2.clone(),
        EnemyType::Stationary => asset_server.t_enemy3.clone(),
        EnemyType::StationaryRanged => asset_server.t_enemy4.clone(),
        EnemyType::Summoner => asset_server.t_enemy5.clone(),
        _ => asset_server.t_temp.clone(),
    };

    commands
        .spawn((
            SpriteBundle {
                texture: t_texture,
                sprite: Sprite {
                    custom_size: Some(Vec2::new(e_size * 2.0, e_size * 2.0)),
                    ..default()
                },
                transform: Transform::from_xyz(e_pos.x, e_pos.y, -1.0),
                ..Default::default()
            },
            // position and enemy info
            //TransformBundle::from(Transform::from_xyz(e_pos.x, e_pos.y, 0.0)),
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

#[allow(dead_code, unreachable_patterns)]
/// spawns an projectile 
pub fn ranged_enemy_shoot(
    commands: &mut Commands,
    p_pos: Vec2,
    p_dir: Vec2,
    p_type: ProjectileType,
    //e_type: EnemyInfo,
    asset_server: &Res<SceneAsset>,
) {
    //TODO add other types of projectiles so I can change what they look like
    let t_texture = match p_type {
        ProjectileType::Contact(_) => asset_server.t_enemy_p.clone(),
        _ => asset_server.t_temp.clone(),
    };

    //println!("shoot");
    commands
        .spawn((
            SpriteBundle {
                texture: t_texture,
                sprite: Sprite {
                    custom_size: Some(Vec2::new(p_type.get_size() * 2.0, p_type.get_size() * 2.0)),
                    ..default()
                },
                transform: Transform::from_xyz(p_pos.x, p_pos.y, -1.0),
                ..Default::default()
            },
            // from data provided
            //TransformBundle::from(Transform::from_xyz(p_pos.x, p_pos.y, 0.0)),
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

//idk why I made this, I had a spawn enemy

// #[allow(dead_code, unreachable_patterns)]
// /// spawns an enemy
// pub fn summon_enemy(
//     commands: &mut Commands,
//     e_pos: Vec2,
//     e_info: EnemyInfo,
//     e_size: f32,
//     asset_server: &Res<SceneAsset>,
// ) {
//     // this sets the texture based on what the enemy is
//     let t_texture = match e_info.e_type {
//         EnemyType::RunDown => asset_server.t_enemy.clone(),
//         EnemyType::Ranged => asset_server.t_enemy2.clone(),
//         EnemyType::Stationary => asset_server.t_enemy3.clone(),
//         EnemyType::StationaryRanged => asset_server.t_enemy4.clone(),
//         EnemyType::Summoner => asset_server.t_enemy5.clone(),
//         _ => asset_server.t_temp.clone(),
//     };

//     //println!("shoot");
//     commands
//         .spawn((
//             SpriteBundle {
//                 texture: t_texture,
//                 sprite: Sprite {
//                     custom_size: Some(Vec2::new(e_size * 2.0, e_size * 2.0)),
//                     ..default()
//                 },
//                 transform: Transform::from_xyz(e_pos.x, e_pos.y, -1.0),
//                 ..Default::default()
//             },
//             // position and enemy info
//             //TransformBundle::from(Transform::from_xyz(e_pos.x, e_pos.y, 0.0)),
//             Collider::ball(e_size),
//             e_info,

//             // default settings
//             CollisionGroups::new(
//                 // G1 is Scorch, G2 is embers, G3 is blocks, G4 is enemies, G5 is enemy_projectiles
//                 Group::GROUP_4,
//                 Group::GROUP_1 | Group::GROUP_2 | Group::GROUP_3,
//             ),
//             RigidBody::Dynamic,
//             Restitution::coefficient(0.5),
//             ExternalImpulse::default(),
//             Velocity::default(),
//             GravityScale(0.0),
//             ColliderMassProperties::Density(1.0),
//             LockedAxes::ROTATION_LOCKED,
//             ActiveEvents::COLLISION_EVENTS,
//         ));
// }