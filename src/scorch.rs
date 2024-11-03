use bevy::{ecs::{entity, query::{self, QueryData}}, prelude::*};
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

#[derive(Component)]
#[allow(dead_code)]
pub struct Scorch {
    /// max life for scorch
    pub max_flame: f32,
    /// current life for scorch
    pub curr_flame: f32,

    // pub FlameForce: f32,

    // pub DoubleJump: bool,
    // pub UnlockedDJ: bool,

    // pub Dash: bool,
    // pub UnlockedDash: bool,
    // pub UnlockedAirDash: bool,
}

impl Scorch {
    pub fn regen_flame(&mut self) {
        if self.max_flame > self.curr_flame {
            self.curr_flame += 0.1;
        }
    }
}
const FORCE_STRENGTH: f32 = 99999.9;
const EXTINGUISH_DIST: f32 = 100.0;

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
            },
            ActiveEvents::COLLISION_EVENTS,
            //Friction::coefficient(0.0),
        ));
}

// this handles impulse forces on Scorch
fn propell_scorch(
    mut commands: Commands,
    mut query: Query<(&mut ExternalImpulse, &Transform, &mut Scorch), With<Scorch>>,
    // idk if this with scorch is needed (aka if it just gets all the pos and impu otherwise)
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut rng: ResMut<RngResource>,
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
        if let Ok(window) = q_windows.get_single() {
            if let Some(mut position) = window.cursor_position() {
                position -= Vec2::new(window.width() * 0.5, window.height() * 0.5);
                //the y coord come out reversed compared to the position of the Scorch
                position.y = -position.y;
                for (
                    mut impulse, 
                    transform, 
                    mut player
                ) in query.iter_mut() {
                    // the camera is locked y wise but x wise its tracking the main character's x 
                    // so you only need to consiter the difference in y and the x position of the mouse
                    // if I locked the y to the charater then I would only need to consiter the mouse position
                    let imp_dir = Vec2::new(
                        -position.x, 
                        transform.translation.y - position.y
                    ).normalize();

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
    mut entity_properties: Query<(&mut ExternalImpulse, &mut Velocity, &mut Transform), With<Scorch>>,
    key_presses: Res<ButtonInput<KeyCode>>,
) {
    //I dont want to waste resources checking if it should move unless one of the keys are being pressed
    if key_presses.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD, KeyCode::Space]) {
        // get the pos and vel of the Scorch
        for (mut imp, mut velo , pos)in entity_properties.iter_mut() {
            // this checks if theres an entity below the shpere within 2m
            if let Some((_entity, _toi)) = &rc.cast_ray(
                Vect::new(pos.translation.x, pos.translation.y - 52.0),
                Vect::new(0.0, 1.0),
                1.9,
                true,
                QueryFilter::default(),
            ) {
                // jump impulse
                if key_presses.just_pressed(KeyCode::Space) {
                    imp.impulse += Vec2::new(0.0, 30.0 * FORCE_STRENGTH);
                    //TODO add double jump
                    // I think that there could be a resource or component to the character,
                    // Then when grounded, this is activated (true) and if used before landing you can jump
                    // When landed it refreshes if (false) but all good because it can only be used when falling
                }
                
                // moveing up. use this when added swimming
                // if key_presses.pressed(KeyCode::KeyW) {
                //     velo.linvel += Vec2::new(0.0, 2.0);
                // }

                //TODO add dashing left and right
            } else {
                //TODO falling
            }
            //TODO for now this allows air strafing and fast falling
            // moving left
            if key_presses.pressed(KeyCode::KeyA) {
                velo.linvel += Vec2::new(-2.0, 0.0);
            }

            // fast falling
            if key_presses.pressed(KeyCode::KeyS) {
                velo.linvel += Vec2::new(0.0, -2.0);
            }

            // moving right
            if key_presses.pressed(KeyCode::KeyD) {
                velo.linvel += Vec2::new(2.0, 0.0);
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