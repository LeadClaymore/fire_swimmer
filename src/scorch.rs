use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use rand::Rng;
// use rand::SeedableRng;
// use rand::rngs::SmallRng;
use bevy::window::PrimaryWindow;

// stuff elsewhere in the project
use crate::{blocks::BlockInfo, coll::DebugComp, ember::EmberComponent, rng::RngResource};

impl Plugin for ScorchPlugin {
    fn build(&self, app: &mut App) {
        // graphical and underlying stuff
        app
            .add_systems(Startup, setup_physics)
            .add_systems(Update, propell_scorch)
            .add_systems(Update, character_movement)
            .add_systems(PostUpdate, restart_scorch)
            .add_systems(PostUpdate, regen_flame)
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

#[derive(Component)]
#[allow(dead_code)]
pub struct Scorch {
    /// max life for scorch
    pub max_flame: f32,
    /// current life for scorch
    pub curr_flame: f32,

    // pub flame_force: f32,

    pub double_jump: bool,
    // pub unlocked_dj: bool,

    pub dash: (bool, f32),

    /// if dash is available and when last pressed
    pub a_dash: f32,
    
    /// if dash is available and when last pressed
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
            curr_time - self.a_dash > DOUBLE_TAP_COOLDOWN && 
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
            curr_time - self.d_dash > DOUBLE_TAP_COOLDOWN &&
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

fn setup_physics(mut commands: Commands) {
    // this is the Scorch
    commands
        .spawn((
            RigidBody::Dynamic,
            Collider::ball(50.0),
            Restitution::coefficient(0.7),
            TransformBundle::from(Transform::from_xyz(0.0, 400.0, 0.0)),
            ExternalImpulse::default(),
            Velocity::default(),
            GravityScale(0.5),
            ColliderMassProperties::Density(1.0),
            LockedAxes::ROTATION_LOCKED,
            Damping {linear_damping: 0.1, angular_damping: 0.0},
            Scorch {
                max_flame: 100.0,
                curr_flame: 100.0,
                double_jump: false,
                dash: (false, 0.0),
                a_dash: 0.0,
                d_dash: 0.0,
            },
            ActiveEvents::COLLISION_EVENTS,
            //Friction::coefficient(0.0),
        ));
}

// this handles impulse forces on Scorch
fn propell_scorch(
    mut commands: Commands,
    //TODO idk if this with scorch is needed (aka if it just gets all the pos and impu otherwise)
    mut query: Query<(&mut ExternalImpulse, &Transform, &mut Scorch), With<Scorch>>,

    //window and camera stuff
    q_windows: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    
    //input
    mouse_input: Res<ButtonInput<MouseButton>>,

    //rng for spawning embers
    mut rng: ResMut<RngResource>,
    // for raycasting for exstinguishing
    rc: Res<RapierContext>,

    // query for blocks with block info for extinguish
    mut bi_query: Query<&mut BlockInfo>,
) {
    // these imputs are used elsewhere so im storing this now
    let (left_click, right_click) = (
        mouse_input.pressed(MouseButton::Left), 
        mouse_input.pressed(MouseButton::Right)
    );
    if left_click || right_click {
        // checks if the window exists and works
        if let Ok(window) = q_windows.get_single() {
            // gets the cursor positon on the window
            if let Some(mut position) = window.cursor_position() {
                // get the camera and its transform for getting its position
                if let Ok((camera, camera_trans)) = q_camera.get_single() {
                    // if the camera position can be translated to world coords
                    if let Some(camera_world)= camera.ndc_to_world(camera_trans, Vec3::ZERO) {
                        // get the forces transform and the scorch info on the player
                        for (
                            mut impulse, 
                            transform, 
                            mut player
                        ) in query.iter_mut() {
                            println!("");
                            print!("pos 0: {}. ", position);
                            //modifications to the position of the mouse to set to where compared to the center
                            //the y coord come out reversed compared to the position of the Scorch
                            position -= Vec2::new(
                                window.width() * 0.5, 
                                window.height() * 0.5
                            );
                            print!("pos 1: {}. ", position);
                            position += Vec2::new(camera_world.x, camera_world.y);
                            print!("pos 2: {}. ", position);
                            position.y = -position.y;
                            print!("pos 3: {}. ", position);
                            print!("S pos: {}. ", transform.translation);

                            // the camera is locked y wise but x wise its tracking the main character's x 
                            // so you only need to consiter the difference in y and the x position of the mouse
                            // if I locked the y to the charater then I would only need to consiter the mouse position
                            let imp_dir = Vec2::new(
                                transform.translation.x - position.x, 
                                transform.translation.y - position.y
                            ).normalize();
        
                            //TODO delete 
                            print!("dir: {}. ", imp_dir);
        
                            // if left click, and has fuel then propell scorch
                            if player.curr_flame > 0.0 && left_click {
                                // apply force
                                impulse.impulse = imp_dir * FORCE_STRENGTH;
        
                                // spawn particle
                                commands.spawn((
                                    EmberComponent::full(),
                                    RigidBody::Dynamic,
                                    Collider::ball(5.0),
                                    Restitution::coefficient(0.7),
                                    TransformBundle::from(Transform::from_xyz(
                                        transform.translation.x - imp_dir.x * 60.0, 
                                        transform.translation.y - imp_dir.y * 60.0, 
                                        1.0
                                    )),
                                    ExternalImpulse {
                                        impulse: Vec2::new(
                                            -imp_dir.x + rng.rng.gen_range(-0.5..0.5),
                                            -imp_dir.y + rng.rng.gen_range(-0.5..0.5),
                                        ) * FORCE_STRENGTH,
                                        torque_impulse: 0.0,
                                    },
                                ));
        
                                // for using up the flame the charater has
                                // with this setup its posible to go negative flame, tbh IDC if that happens
                                player.curr_flame -= 1.0;
        
                                /*
                                    spawn through EmberComponent
                                    EmberComponent::spawn_ember(
                                        &commands, 
                                        (
                                                transform.translation.x - imp_dir.x * 60.0,
                                                transform.translation.y - imp_dir.y * 60.0
                                            ), 
                                        Vec2::new(
                                                -imp_dir.x + rng.rng.gen_range(-0.5..0.5),
                                                -imp_dir.y + rng.rng.gen_range(-0.5..0.5),
                                            ) * FORCE_STRENGTH
                                    );
                                */
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
                            ));
                        }
                    }
                }
            }
        }
    }
}

// when the R key is pressed it resets it to the starting position
fn restart_scorch(
    mut entity_phys: Query<(&mut ExternalImpulse, &mut Velocity, &mut Transform), With<Scorch>>,
    key_presses: Res<ButtonInput<KeyCode>>,
) {
    if key_presses.just_pressed(KeyCode::KeyR) {
        for (mut impulse, mut velocity, mut position) in entity_phys.iter_mut() {
            position.translation = Vec3::new(0.0, 400.0, 0.0);
            impulse.impulse = Vec2::ZERO;
            velocity.linvel = Vec2::ZERO;
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
                    println!("double jump!");
                    imp.impulse += Vec2::new(0.0, 30.0 * FORCE_STRENGTH);
                }
                //TODO look up if having the checks and times for dashing and double jumping would be better as a resource
            }
            //TODO for now this allows air strafing and fast falling
            // moving left
            if key_presses.just_pressed(KeyCode::KeyA) && scorch.a_dash_avail(time.elapsed_seconds()){
                //println!("a dash!");
                imp.impulse += Vec2::new(-50.0 * FORCE_STRENGTH, 10.0 * FORCE_STRENGTH );
                velo.linvel.y = 0.0;
            } else if key_presses.pressed(KeyCode::KeyA) {
                velo.linvel += Vec2::new(-2.0, 0.0);
            }

            if key_presses.just_pressed(KeyCode::KeyD) && scorch.d_dash_avail(time.elapsed_seconds()){
                //println!("d dash!");
                imp.impulse += Vec2::new(50.0 * FORCE_STRENGTH, 10.0 * FORCE_STRENGTH );
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