use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

const FORCE_STRENGTH: f32 = 99999.9;

#[derive(Component)]
enum Flame_Strngth {
    Dead,
    Weak,
    Normal,
    Strong,
    Full,
}

#[derive(Component)]
struct Ball;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, (setup_graphics, setup_physics))
        // TODO move to a scheduling system
        // movement of the ball 
        .add_systems(Update, (propell_ball, restart_ball))
        .add_systems(Update, despawn_particles)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // this is the default camera
    commands.spawn(Camera2dBundle::default());
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

// this handles impulse forces on the ball
fn propell_ball(
    mut commands: Commands,
    mut query: Query<(&mut ExternalImpulse, &Transform), With<Ball>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    if mouse_input.pressed(MouseButton::Left) {
        if let Ok(window) = q_windows.get_single() {
            if let Some(mut position) = window.cursor_position() {
                position -= Vec2::new(window.width() * 0.5, window.height() * 0.5);
                //the y coord come out reversed compared to the position of the ball
                position.y = -position.y;
                for (mut impulse, transform) in query.iter_mut() {
                    let the_impulse = (Vec2::new(transform.translation.x, transform.translation.y) - position)
                    .normalize();
                    impulse.impulse = the_impulse * FORCE_STRENGTH;
                    //impulse.torque_impulse = 1.0;
                    //println!("cursor: {:?}, ball: {:?}, force: {:?}", position, transform.translation, the_impulse);

                    //spawn particle
                    commands.spawn((
                        Flame_Strngth::Full,
                        RigidBody::Dynamic,
                        Collider::ball(5.0),
                        Restitution::coefficient(0.7),
                        TransformBundle::from(Transform::from_xyz(
                            transform.translation.x - the_impulse.x * 100.0, 
                            transform.translation.y - the_impulse.y * 100.0, 
                            1.0
                        )),
                        ExternalImpulse {
                            impulse: -the_impulse * FORCE_STRENGTH,
                            torque_impulse: 0.0,
                        },
                    ));
                    println!("spawned flame");
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

// despawn particles currently on keypress
fn despawn_particles (
    mut commands: Commands,
    query: Query<Entity, With<Flame_Strngth>>,
    key_press: Res<ButtonInput<KeyCode>>,
) {
    if key_press.just_pressed(KeyCode::KeyL) {
        for entity in query.iter() {
            commands.entity(entity).despawn();
        }
    }
}