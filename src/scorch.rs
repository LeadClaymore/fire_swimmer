use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use rand::Rng;
// use rand::SeedableRng;
// use rand::rngs::SmallRng;
use bevy::window::PrimaryWindow;

// stuff elsewhere in the project
use crate::{
    asset_loader::SceneAsset, 
    blocks::BlockInfo, 
    coll::DebugComp, 
    ember, 
    rng::RngResource,
    state_system::AppState,
};

impl Plugin for ScorchPlugin {
    fn build(&self, app: &mut App) {
        // graphical and underlying stuff
        app
            .add_systems(
                OnEnter(AppState::InGame), 
                setup_physics
            )
            .add_systems(
                Update, 
                (propell_scorch, character_movement).run_if(in_state(AppState::InGame))
            )
            .add_systems(
                PostUpdate, 
                (restart_scorch, regen_flame).run_if(in_state(AppState::InGame))
            )
        ;
    }
}

// stuff in here
pub struct ScorchPlugin;

#[derive(Bundle)]
pub struct ScorchBundle {
    pub scorch: Scorch,
}

// TODO change the physics of the entier game to mean this does not need to be 100k
/// a modifier added to impulses to move the charater
const FORCE_STRENGTH: f32 = 99999.9;

/// How far you can extinguish a block from
const EXTINGUISH_DIST: f32 = 100.0;

//TODO move to charater feature if it can be changed
/// how long between dashes
const DASH_COOLDOWN: f32 = 1.0;

/// How long between presses would make a dash or something else
const DOUBLE_TAP_COOLDOWN: f32 = 0.2;

#[derive(Component, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct Scorch {
    /// max life for scorch
    pub max_flame: f32,
    /// current life for scorch
    pub curr_flame: f32,

    /// the time that you are invincable
    pub i_frame: f32,
    /// the last time you were hit
    pub i_frame_timer: f32,

    // pub flame_force: f32,

    pub double_jump: bool,
    // pub unlocked_dj: bool,

    /// if you have the dash avaliable
    /// and when the last dash was
    pub dash: (bool, f32),
    /// When a was last pressed
    pub a_dash: f32,
    /// When d was last pressed
    pub d_dash: f32,

    // pub unlocked_dash: bool,
    // pub unlocked_air_dash: bool,
}

impl Scorch {
    pub fn regen_flame(&mut self) {
        if self.max_flame > self.curr_flame {
            self.curr_flame += 0.1;
        }
    }

    ///Take damage based on the send number
    pub fn damage_flame(&mut self, dmg: f32, curr_time: f32) -> bool {
        if self.i_frame + self.i_frame_timer < curr_time {
            self.i_frame_timer = curr_time;
            //println!("damage taken!");

            if dmg < self.curr_flame {
                self.curr_flame -= dmg;
            } else {
                self.curr_flame = 0.0;
            }
            return true;
        } else {
            //print!("blocked");
            return false;
        }
    }

    ///reset scorch stats
    pub fn reset(&mut self) {
        // currently just flame
        self.curr_flame = self.max_flame;
    }

    pub fn grounded(&mut self) {
        self.dash.0 = true;
        self.double_jump = true;
    }

    /// if the A was pressed within the last .2 sec and dash is available, then set dash to false and return true.
    /// otherwise record curr time in a_dash.1
    pub fn a_dash_avail(&mut self, curr_time: f32) -> bool {
        // TODO currently a_dash.1 starts as 0, so insta dash
        // fix by checking for == 0.0
        if self.dash.0 && 
            curr_time - self.a_dash < DOUBLE_TAP_COOLDOWN && 
            curr_time - self.dash.1 > DASH_COOLDOWN
        {
            self.dash.0 = false;
            self.dash.1 = curr_time;
            return true;
        }
        self.a_dash = curr_time;
        return false;
    }

    /// if the D was pressed within the last .2 sec and dash is available, then set dash to false and return true.
    /// otherwise record curr time in d_dash.1
    pub fn d_dash_avail(&mut self, curr_time: f32) -> bool {
        if 
            self.dash.0 && 
            curr_time - self.d_dash < DOUBLE_TAP_COOLDOWN &&
            curr_time - self.dash.1 > DASH_COOLDOWN 
        {
            self.dash.0 = false;
            self.dash.1 = curr_time;
            return true;
        }
        self.d_dash = curr_time;
        return false;
    }

    /// if the double jump is available then return true and set it to false
    /// otherwise return false;
    pub fn double_jump_avail(&mut self) -> bool {
        if self.double_jump {
            self.double_jump = false;
            return true;
        }
        return false;
    }
}


#[derive(Component, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct DetectRange;

fn setup_physics(
    mut commands: Commands,
    asset_server: Res<SceneAsset>,
) {
    //let t_scorch = asset_server.load("assets/t_block.png");

    // this is the Scorch
    commands
        .spawn((
            RigidBody::Dynamic,
            Collider::ball(50.0),
            CollisionGroups::new(
                // G1 is Scorch, G2 is embers, G3 is blocks, G4 is enemies, G5 is enemy_projectiles
                Group::GROUP_1,
                Group::GROUP_2 | Group::GROUP_3 | Group::GROUP_4 | Group::GROUP_5,
            ),
            ActiveEvents::COLLISION_EVENTS,
            Restitution::coefficient(0.1),
            //TransformBundle::from(Transform::from_xyz(0.0, 0.0, 0.0)),
            ExternalImpulse::default(),
            Velocity::default(),
            GravityScale(0.5),
            ColliderMassProperties::Density(1.0),
            LockedAxes::ROTATION_LOCKED,
            SpriteBundle {
                texture: asset_server.t_scorch.clone(),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(100.0, 100.0)),
                    ..default()
                },
                transform: Transform::from_xyz(0.0, 0.0, -1.0),
                ..Default::default()
            },
            Damping {
                linear_damping: 0.1, 
                angular_damping: 0.0
            },
            Scorch {
                max_flame: 100.0,
                curr_flame: 100.0,

                i_frame: 1.0,
                i_frame_timer: 0.0,

                double_jump: false,

                dash: (false, 0.0),
                a_dash: 0.0,
                d_dash: 0.0,
            },
        ))
        .with_children(|parent| {
            // Add the sensor collider as a child
            parent
                .spawn( (
                    DetectRange, //This is for collisions to determin if they should be in range
                    Collider::ball(1000.0),
                    Sensor,
                    CollisionGroups::new(
                        //TODO for now it collides with nothing
                        // G30 is going to be debug objects
                        Group::GROUP_1,
                        Group::GROUP_4
                    ),
                    ActiveEvents::COLLISION_EVENTS,
                    ColliderMassProperties::Mass(0.0),
                    TransformBundle::default(), // Ensure it follows the parent
                ));
                
        })
        ;
}

// this handles impulse forces on Scorch
fn propell_scorch(
    mut commands: Commands,
    // transform of scorch
    mut query: Query<(&mut ExternalImpulse, &Transform, &mut Scorch)>,
    // window and camera stuff
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    // input
    mouse_input: Res<ButtonInput<MouseButton>>,
    // rng for spawning embers
    mut rng: ResMut<RngResource>,
    // for raycasting for exstinguishing
    rc: Res<RapierContext>,
    // query for blocks with block info for extinguish
    mut bi_query: Query<&mut BlockInfo>,
    //for spawning embers with the right texture
    asset_server: Res<SceneAsset>,
) {
    // these imputs are used elsewhere so im storing this now
    let (left_click, right_click) = (
        mouse_input.pressed(MouseButton::Left), 
        mouse_input.pressed(MouseButton::Right)
    );
    if left_click || right_click {
        // I found this example in the bevy cookbook
        // https://bevy-cheatbook.github.io/cookbook/cursor2world.html
        // this gets the camera and the transform and window
        let (camera, camera_transform) = q_camera.single();
        let window = q_window.single();
        // this gets the window by getting the cursor pos on screen, (.cursor_position)
        // convert that pos on screen to a transform pos and dir in ray3d, (.viewport_to_world)
        // then gets the transform from the ray3d, (.orgin)
        // then discards the z using truncate, (.truncate)
        // tada a vec2 of the position of the camera
        // god I wish I knew about these functions before
        if let Some(world_position) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            // get the forces transform and the scorch info on the player
            for (
                mut impulse, 
                transform, 
                mut player
            ) in query.iter_mut() {
                // the camera is locked y wise but x wise its tracking the main character's x 
                // so you only need to consiter the difference in y and the x position of the mouse
                // if I locked the y to the charater then I would only need to consiter the mouse position
                let imp_dir = Vec2::new(
                    transform.translation.x - world_position.x, 
                    transform.translation.y - world_position.y
                ).normalize();
                // if left click, and has fuel then propell scorch
                if player.curr_flame > 0.0 && left_click {
                    // apply force
                    impulse.impulse = imp_dir * FORCE_STRENGTH;

                    // spawn particle
                    ember::spawn_ember(
                        &mut commands, 
                        &asset_server,
                        (
                            transform.translation.x - imp_dir.x * 60.0, 
                            transform.translation.y - imp_dir.y * 60.0
                        ),
                        Vec2::new(
                            -imp_dir.x + rng.rng.gen_range(-0.5..0.5),
                            -imp_dir.y + rng.rng.gen_range(-0.5..0.5),
                        ) * FORCE_STRENGTH
                    );

                    // for using up the flame the charater has
                    // with this setup its posible to go negative flame, tbh IDC if that happens
                    player.curr_flame -= 1.0;
                } else if right_click {
                    if let Some((ext_entity, _toi)) = &rc.cast_ray(
                    Vect::new(
                        transform.translation.x + imp_dir.x * 60.0, 
                        transform.translation.y + imp_dir.y * 60.0
                    ),
                    Vect::new(imp_dir.x, imp_dir.y),
                    EXTINGUISH_DIST,
                    true,
                    QueryFilter::default(),
                    ) {
                        // if let check for the block having block info
                        if let Ok(mut ex_block) = bi_query.get_mut(*ext_entity) {
                            if ex_block.extinguishable && ex_block.burn_time.1 != 0.0 {
                                //println!("extinguish block!");
                                ex_block.burn_time.1 = 0.0;
                            } else if ex_block.extinguishable {
                                //println!("block not on fire!");
                            } else {
                                //println!("not valid target!");
                            }
                        } else {
                            //println!("not a block!");
                        }
                    }
                }

                //drawing debug line
                // Define the start and end points of the line
                let start = Vect::new(
                    transform.translation.x, 
                    transform.translation.y
                );
                let end = Vect::new(
                    transform.translation.x - imp_dir.x * 1000.0, 
                    transform.translation.y - imp_dir.y * 1000.0
                );

                // Create a mesh with two vertices
                commands
                    .spawn((
                    RigidBody::Fixed,
                    DebugComp,
                    Collider::segment(start, end),
                    Sensor,
                    //TODO I expected and error with this debug object colliding
                    CollisionGroups::new(
                        // G30 is going to be debug objects
                        Group::GROUP_30,
                        Group::NONE
                    ),
                ));
            }
        }
    }
}

#[allow(irrefutable_let_patterns, dead_code, unused_mut)]
/// when the R key is pressed it resets it to the starting position
fn restart_scorch(
    mut commands: Commands,
    mut s_query: Query<(&mut Scorch, &mut ExternalImpulse, &mut Velocity, &mut Transform)>,
    key_presses: Res<ButtonInput<KeyCode>>,
    asset_server: Res<SceneAsset>, //TODO make functions not need to call this
) {
    // let (s_entity, mut s_compo) = scor_query.single_mut();
    if key_presses.just_pressed(KeyCode::KeyR) {
        // I swear if scorch DNE then this should fail just in case I will leave this, and add the allow
        if let (
            mut s_info, 
            mut s_impulse, 
            mut s_velocity, 
            mut s_position
        ) = s_query.single_mut() {
            s_position.translation = Vec3::new(0.0, 0.0, -1.0);
            s_impulse.impulse = Vec2::ZERO;
            s_velocity.linvel = Vec2::ZERO;
            s_info.reset();
        } else {
            setup_physics(commands, asset_server);
        }
    }
}

fn character_movement(
    rc: Res<RapierContext>,
    mut entity_properties: Query<(&mut ExternalImpulse, &mut Velocity, &mut Transform, &mut Scorch)>,
    key_presses: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    //I dont want to waste resources checking if it should move unless one of the keys are being pressed
    if key_presses.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD, KeyCode::Space]) {
        // get the pos and vel of the Scorch
        for (mut imp, mut velo , pos, mut scorch)in entity_properties.iter_mut() {
            // this checks if theres an entity below the shpere within 2m
            if let Some((_entity, _toi)) = &rc.cast_ray(
                Vect::new(pos.translation.x, pos.translation.y - 52.0),
                Vect::new(0.0, 1.0),
                1.9,
                true,
                QueryFilter::default(),
            ) {
                scorch.grounded();
                // jump impulse
                if key_presses.just_pressed(KeyCode::Space) {
                    imp.impulse += Vec2::new(0.0, 30.0 * FORCE_STRENGTH);
                }
                
                // moveing up. use this when added swimming
                // if key_presses.pressed(KeyCode::KeyW) {
                //     velo.linvel += Vec2::new(0.0, 2.0);
                // }

            } else {
                //falling

                // doublejump if possible
                if key_presses.just_pressed(KeyCode::Space) && scorch.double_jump_avail() {
                    //println!("double jump!");
                    imp.impulse += Vec2::new(0.0, 30.0 * FORCE_STRENGTH);
                    //TODO add some VFX on double jump also maybe add a flame cost
                }
            }
            // moving left
            if key_presses.just_pressed(KeyCode::KeyA) && scorch.a_dash_avail(time.elapsed_seconds()){
                imp.impulse += Vec2::new(-50.0 * FORCE_STRENGTH, 5.0 * FORCE_STRENGTH );
                velo.linvel.y = 0.0;
            } else if key_presses.pressed(KeyCode::KeyA) {
                velo.linvel += Vec2::new(-2.0, 0.0);
            }

            if key_presses.just_pressed(KeyCode::KeyD) && scorch.d_dash_avail(time.elapsed_seconds()){
                imp.impulse += Vec2::new(50.0 * FORCE_STRENGTH, 5.0 * FORCE_STRENGTH );
                velo.linvel.y = 0.0;
            } else if key_presses.pressed(KeyCode::KeyD) {
                velo.linvel += Vec2::new(2.0, 0.0);
            }

            // fast falling
            if key_presses.pressed(KeyCode::KeyS) {
                velo.linvel += Vec2::new(0.0, -2.0);
            }
        }
    }
}

fn regen_flame (
    mut query: Query<&mut Scorch>,
) {
    for mut scorch in query.iter_mut() {
        scorch.regen_flame();
    }
}
// end