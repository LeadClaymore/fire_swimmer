use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

use bevy::window::PrimaryWindow;
//use bevy::render::camera::Viewport;

const FORCE_STRENGTH: f32 = 99999.9;

//TODO tbh I might change this out for a basic timer idk
/// a resource for calculating when flames should die down
#[derive(Resource)]
pub struct EmberTimer(Timer);

/// This is an struct for information on the burn type for a block
#[derive(Component, Debug, Clone, Copy)]
pub struct BlockInfo {
    /// If this can be set on fire
    pub burnable:       bool,
    /// When burnt can it be put out
    pub extinguishable: bool,
    /// (How long it will burn, when it starts burning (preburn == f64::MAX))
    pub burn_time:      (f32, f32),
    // currently dont use pos and size
    // /// position of the block
    // pub pos:            Vec2,
    // /// size of the block
    // pub size:           Vec2,
    //TODO slants, movable, explosive
}

impl BlockInfo {
    fn default(self) -> BlockInfo {
        BlockInfo {
            burnable:       true,
            extinguishable: true,
            burn_time:      (10.0, f32::MAX),
        }
    }
    
    fn new(burn: bool, exti: bool, btime: f32) -> BlockInfo {
        BlockInfo {
            burnable:       burn,
            extinguishable: exti,
            burn_time:      (btime, f32::MAX),
        }
    }
    
    fn set_burn(mut self, start_time: f32) {
        self.burn_time.1 = start_time;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FlameStrength {
    Weak,
    Normal,
    Strong,
    Full,
}

#[derive(Component)]
pub struct FlameComponent {
    pub state: FlameStrength,
}

#[derive(Component)]
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

#[derive(Component)]
pub struct MainCamera;

#[derive(Resource)]
pub struct RngResource {
    pub rng: rand::rngs::SmallRng,
}

impl Default for RngResource {
    fn default() -> Self {
        Self { rng: SmallRng::from_entropy(), }
    }
}

fn setup_graphics(mut commands: Commands) {
    // this is the default camera
    commands.spawn((
        Camera2dBundle {
            projection: OrthographicProjection {
                scale: 2.0,
                ..default()
            },
            ..default()
        },
        MainCamera,
    ));
}

fn setup_physics(mut commands: Commands) {
    // this is the platform 
    commands
        .spawn((
            // RigidBody::Dynamic,
            Collider::cuboid(500.0, 50.0),
            TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)),
            BlockInfo::new(
                false, 
                false, 
                5.0, 
            ),
            //Friction::coefficient(2.0),
        ));

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
            //Friction::coefficient(0.0),
            Scorch {
                max_flame: 100.0,
                curr_flame: 100.0,
            },
        ));
}

// camera will follow the x axis of the Scorch
fn camera_control(
    character_query: Query<&Transform, With<Scorch>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Scorch>)>,
) {
    // learning moment, even though there are no transforms with MainCamera and Scorch, 
    // when we are querying one to be mutable and the other immutable,
    // we need to the query of transforms with MainCamera does not contain Scorch 
    // because we cant query the same component one mutable and the other not
    if let Ok(character_transform) = character_query.get_single() {
        if let Ok(mut camera_transform) = camera_query.get_single_mut() {
            camera_transform.translation.x = character_transform.translation.x;
        } else {
            //println!("ERROR! camera transform unable to parse");
        }
    } else {
        //println!("ERROR! character transform unable to parse");
    }
}


// this handles impulse forces on Scorch
fn propell_scorch(
    mut commands: Commands,
    mut query: Query<(&mut ExternalImpulse, &Transform, &mut Scorch), With<Scorch>>,
    // idk if this with scorch is needed (aka if it just gets all the pos and impu otherwise)
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut rng: ResMut<RngResource>,
) {
    if mouse_input.pressed(MouseButton::Left) {
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
                    if player.curr_flame > 0.0 {
                        // the camera is locked y wise but x wise its tracking the main character's x 
                        // so you only need to consiter the difference in y and the x position of the mouse
                        // if I locked the y to the charater then I would only need to consiter the mouse position
                        let the_impulse = Vec2::new(-position.x, transform.translation.y - position.y).normalize();
                        
                        impulse.impulse = the_impulse * FORCE_STRENGTH;
                        //impulse.torque_impulse = 1.0;
                        //println!("cursor: {:?}, Scorch: {:?}, force: {:?}", position, transform.translation, the_impulse);

                        //spawn particle
                        commands.spawn((
                            FlameComponent {
                                state: FlameStrength::Full,
                            },
                            RigidBody::Dynamic,
                            Collider::ball(5.0),
                            Restitution::coefficient(0.7),
                            TransformBundle::from(Transform::from_xyz(
                                transform.translation.x - the_impulse.x * 60.0, 
                                transform.translation.y - the_impulse.y * 60.0, 
                                1.0
                            )),
                            ExternalImpulse {
                                impulse: Vec2::new(
                                    -the_impulse.x + rng.rng.gen_range(-0.5..0.5),
                                    -the_impulse.y + rng.rng.gen_range(-0.5..0.5),
                                ) * FORCE_STRENGTH,
                                torque_impulse: 0.0,
                            },
                        ));
                        //println!("spawned flame");
                        player.curr_flame -= 1.0;
                        // with this setup its posible to go negative flame, tbh IDC if that happens
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

// the embers fade every tick of the ember timer, when they reach the end they despawn
fn despawn_particles (
    mut commands: Commands,
    mut query: Query<(Entity, &mut FlameComponent)>,
    //key_press: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ember_timer: ResMut<EmberTimer>,
) {
    if ember_timer.0.tick(time.delta()).just_finished() {
        for (entity, mut flame) in query.iter_mut() {
            match flame.state {
                FlameStrength::Full => flame.state = FlameStrength::Strong,
                FlameStrength::Strong => flame.state = FlameStrength::Normal,
                FlameStrength::Normal => flame.state = FlameStrength::Weak,
                FlameStrength::Weak => commands.entity(entity).despawn(),
            }
        }
    }
}

//TODO I dont need mut for transform rn, but IDK how to to iter_mut with 1 mut and another not
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
            //TODO make a filter for components With<BlockInfo>
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

fn block_burning_system (
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &BlockInfo)>
) {
    let current_time = time.elapsed_seconds();
    for (entity, info) in query.iter() {
        if info.burn_time.1 != f32::MAX {
            if current_time - info.burn_time.1 > info.burn_time.0 {
                //TODO for now it just despawns, later it might do more
                commands.entity(entity).despawn();
                println!("Burn timer started for block!");
            }
        }
    }
}

fn collision_event_system (
    //mut commands: Commands,
    mut collision_events: EventReader<CollisionEvent>,
    time: Res<Time>,
    binfo_query: Query<(Entity, &mut BlockInfo)>,
    ember_query: Query<Entity, With<FlameComponent>>,
    //mut query: Query<(&mut ActiveCollisionTypes, &mut BlockInfo)>,
) {
    for cevent in collision_events.read() {
        match cevent {
            CollisionEvent::Started(ent1, ent2, _) => {
                // check if 
                if let Ok((_block_ent, binfo)) = binfo_query.get(*ent1) {
                    if ember_query.get(*ent2).is_ok() {
                        binfo.set_burn(time.elapsed_seconds());
                    }
                }
                // same but in reverse
                if let Ok((_block_ent, binfo)) = binfo_query.get(*ent2) {
                    if ember_query.get(*ent1).is_ok() {
                        binfo.set_burn(time.elapsed_seconds());
                    }
                }
            }
            CollisionEvent::Stopped(_, _, _) => {
                //currently unused for collisions, but it was in the example
            }
        }
    }

    //TODO for now it just starts burrning rn
    // for (colid, mut binfo) in query.iter_mut() {
    //     if binfo.burn_time.1 == f32::MAX {
    //         binfo.burn_time.1 = time.elapsed_seconds();
    //     }
    // }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .insert_resource(RngResource::default())
        .insert_resource(EmberTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
        .add_systems(Startup, (setup_graphics, setup_physics))
        .add_systems(Update, camera_control)
        // TODO move to a scheduling system
        // movement of the Scorch 
        .add_systems(Update, (block_burning_system, collision_event_system))
        .add_systems(Update, (propell_scorch, restart_scorch))
        .add_systems(Update, character_movement)
        .add_systems(Update, despawn_particles)
        .run();
}
//end