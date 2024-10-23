use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

const FORCE_STRENGTH: f32 = 99999.9;

#[derive(Component)]
struct Ball;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_physics)
        //.add_systems(Update, print_ball_altitude)
        .add_systems(Update, move_ball)
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // this is the default camera
    commands.spawn(Camera2dBundle::default());
}

fn setup_physics(mut commands: Commands) {
    // this is the platform 
    // collider is where the platform will interact with the ball
    // the transform is where its middle will be
    commands
        .spawn((
            // RigidBody::Dynamic,
            Collider::cuboid(500.0, 50.0),
            TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)),
            //Friction::coefficient(0.0),
        ));

    // this is the ball
    // Ridgid body is how this interacts with stuff
    // collider is the radius of the ball
    // restitution is how much bounce it has
    // the transform is where the ball starts
    commands
        .spawn((
            RigidBody::Dynamic,
            Collider::ball(50.0),
            Restitution::coefficient(0.7),
            TransformBundle::from(Transform::from_xyz(0.0, 400.0, 0.0)),
            ExternalImpulse::default(),
            GravityScale(0.5),
            ColliderMassProperties::Density(2.0),
            //Friction::coefficient(0.0),
            Ball,
        ));
}

fn move_ball(
    //mut commands: Commands,
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
                    .normalize() * FORCE_STRENGTH;
                    impulse.impulse = the_impulse;
                    println!("cursor: {:?}, ball: {:?}, force: {:?}", position, transform.translation, the_impulse);
                }
            }
        }
    } else {
        // for (mut impulse, transform) in query.iter_mut() {
        //     impulse.impulse = (Vec2::new(transform.translation.x, transform.translation.y) - position)
        //         .normalize() * FORCE_STRENGTH;
        //     //println!("cursor: {:?}, ball: {:?}, force: {:?}", position - (window_size * 0.5), transform.translation, force_vector);
        // }
    }
}