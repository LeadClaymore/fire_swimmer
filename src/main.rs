use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_rapier2d::prelude::*;

const FORCE_STRENGTH: f32 = 1000.0;

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
        .spawn(Collider::cuboid(500.0, 50.0))
        .insert(TransformBundle::from(Transform::from_xyz(0.0, -100.0, 0.0)));

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
            ExternalForce::default(),
            Ball,
        ));
}

fn move_ball(
    //mut commands: Commands,
    mut query: Query<(&mut ExternalForce, &Transform), With<Ball>>,
    q_windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window_size = Vec2::new(q_windows.single().width(), q_windows.single().height());
    if let Some(position) = q_windows.single().cursor_position() {
        for (mut force, transform) in query.iter_mut() {
            let force_vector = (Vec2::new(
                transform.translation.x - (position.x - window_size.x * 0.5), 
                transform.translation.y - (position.y - window_size.y * 0.5),
            )) * FORCE_STRENGTH;
            force.force = (force_vector).into();
            println!("cursor: {:?}, ball: {:?}, force: {:?}", position - (window_size * 0.5), transform.translation, force_vector);
        }
    }
    // if let Some(position) = q_windows.single().cursor_position() {
    //     for mut force in query.iter_mut() {
    //         // Apply force towards the cursor position
    //         let force_vector = Vec2::new(position.x - 400.0, position.y - 400.0); // example force direction
    //         force.force = (Vec2::new(force_vector.x, force_vector.y) * FORCE_STRENGTH).into();
    //     }
    // }
}

// fn print_ball_altitude(positions: Query<&Transform, With<RigidBody>>) {
//     for transform in positions.iter() {
//         println!("Ball altitude: {}", transform.translation.y);
//     }
// }