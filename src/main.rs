use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;

use bevy::window::PrimaryWindow;
//use bevy::render::camera::Viewport;

const FORCE_STRENGTH: f32 = 99999.9;

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
pub struct MyCamera;

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
        MyCamera,
    ));
}

fn setup_physics(mut commands: Commands) {
    // this is the platform 
    commands
        .spawn((
            // RigidBody::Dynamic,
            Collider::cuboid(500.0, 50.0),
            TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)),
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
    mut camera_query: Query<&mut Transform, (With<MyCamera>, Without<Ball>)>,
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
// learning moment, even though there are no transforms with mycamera and ball, 
// when we are querying one to be mutable and the other immutable,
// we need to the query of transforms with mycamera does not contain ball 
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
    mut positions: Query<&mut Transform, With<Ball>>,
    key_presses: Res<ButtonInput<KeyCode>>,
) {
    if key_presses.just_pressed(KeyCode::KeyR) {
        for mut position in positions.iter_mut() {
            position.translation = Vec3::new(0.0, 400.0, 0.0);
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

//end