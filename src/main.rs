use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

use bevy::window::PrimaryWindow;
//use bevy::render::camera::Viewport;

const FORCE_STRENGTH: f32 = 99999.9;

/// This is an struct for information on the burn type for a block
#[derive(Component, Debug, Clone, Copy)]
pub struct BlockInfo {
    /// If this can be set on fire
    pub burnable:       bool,
    /// When burnt can it be put out
    pub extinguishable: bool,
    /// When burnt how long will it last
    pub burn_time:      f32,
    /// position of the block
    pub pos:            Vec2,
    /// size of the block
    pub size:           Vec2,
    //TODO slants, movable, explosive
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

#[derive(Resource)]
pub struct EmberTimer(Timer);

#[derive(Component)]
pub struct Ball;

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
        // movement of the ball 
        .add_systems(Update, (propell_ball, restart_ball))
        .add_systems(Update, character_movement)
        .add_systems(Update, despawn_particles)
        .run();
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
            BlockInfo {
                burnable:       false,
                extinguishable: false,
                burn_time:      0.0,
                pos:            Vec2::new(0.0, -100.0),
                size:           Vec2::new(500.0, 50.0),
            },
            //Friction::coefficient(2.0),
        ));

    // this is the ball
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
            Ball,
        ));
}

// camera will follow the x axis of the ball
fn camera_control(
    character_query: Query<&Transform, With<Ball>>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Ball>)>,
) {
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
// learning moment, even though there are no transforms with MainCamera and ball, 
// when we are querying one to be mutable and the other immutable,
// we need to the query of transforms with MainCamera does not contain ball 
// because we cant query the same component one mutable and the other not

// this handles impulse forces on the ball
fn propell_ball(
    mut commands: Commands,
    mut query: Query<(&mut ExternalImpulse, &Transform), With<Ball>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut rng: ResMut<RngResource>,
) {
    if mouse_input.pressed(MouseButton::Left) {
        if let Ok(window) = q_windows.get_single() {
            if let Some(mut position) = window.cursor_position() {
                position -= Vec2::new(window.width() * 0.5, window.height() * 0.5);
                //the y coord come out reversed compared to the position of the ball
                position.y = -position.y;
                for (mut impulse, transform) in query.iter_mut() {
                    // the camera is locked y wise but x wise its tracking the main character's x 
                    // so you only need to consiter the difference in y and the x position of the mouse
                    // if I locked the y to the charater then I would only need to consiter the mouse position
                    let the_impulse = Vec2::new(-position.x, transform.translation.y - position.y).normalize();
                    
                    impulse.impulse = the_impulse * FORCE_STRENGTH;
                    //impulse.torque_impulse = 1.0;
                    //println!("cursor: {:?}, ball: {:?}, force: {:?}", position, transform.translation, the_impulse);

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
                }
            }
        }
    }
}

// when the R key is pressed it resets it to the starting position
fn restart_ball(
    mut entity_phys: Query<(&mut ExternalImpulse, &mut Velocity, &mut Transform), With<Ball>>,
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
    mut entity_properties: Query<(&mut Velocity, &mut Transform), With<Ball>>,
    key_presses: Res<ButtonInput<KeyCode>>,
) {
    //I dont want to waste resources checking if it should move unless one of the keys are being pressed
    if key_presses.any_pressed([KeyCode::KeyW, KeyCode::KeyA, KeyCode::KeyS, KeyCode::KeyD, KeyCode::Space]) {
        // get the pos and vel of the ball
        for (mut velo , pos)in entity_properties.iter_mut() {
            // this checks if theres an entity below the shpere within 2m
            //TODO make a filter for components With<BlockInfo>
            if let Some((_entity, _toi)) = &rc.cast_ray(
                Vect::new(pos.translation.x, pos.translation.y - 52.0),
                Vect::new(0.0, 1.0),
                1.9,
                true,
                QueryFilter::default(),
            ) {
                // TODO Either add this to WKey or move to inpulse
                if key_presses.pressed(KeyCode::Space) {
                    velo.linvel += Vec2::new(0.0, 2.0);
                }
                
                // W key
                if key_presses.pressed(KeyCode::KeyW) {
                    velo.linvel += Vec2::new(0.0, 2.0);
                }
            } else {
                //TODO falling
            }
            //TODO for now this allows air strafing and fast falling
            // moving left
            if key_presses.pressed(KeyCode::KeyA) {
                velo.linvel += Vec2::new(-2.0, 2.0);
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
//end