use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Bundle)]
pub struct EmberBundle {
    //pub ember_str: EmberStrength,
    pub ember_component: EmberComponent,
}

pub struct EmberPlugin;

impl Plugin for EmberPlugin {
    fn build(&self, app: &mut App) {
        // graphical and underlying stuff
        app
            .insert_resource(EmberTimer(Timer::from_seconds(1.0, TimerMode::Repeating)))
            .add_systems(Update, despawn_particles)
            //.add_systems(spawn_ember)
        ;
    }
}

//TODO tbh I might change this out for a basic timer idk
/// a resource for calculating when flames should die down
#[derive(Resource)]
pub struct EmberTimer(Timer);

#[derive(Debug, Clone, Copy)]
pub enum EmberStrength {
    Weak,
    Normal,
    Strong,
    Full,
}

#[derive(Component)]
pub struct EmberComponent {
    pub state: EmberStrength,
}

#[allow(dead_code)]
impl EmberComponent {
    pub fn full() -> Self {
        EmberComponent { 
            state: EmberStrength::Full,
        }
    }

    pub fn spawn_ember(
        commands: &mut Commands,
        pos: (f32, f32), 
        imp: Vec2
    ) {
        // spawn particle
        commands.spawn((
            EmberComponent {
                state: EmberStrength::Full,
            },
            RigidBody::Dynamic,
            Collider::ball(5.0),
            Restitution::coefficient(0.7),
            TransformBundle::from(Transform::from_xyz(
                pos.0, 
                pos.1, 
                1.0
            )),
            ExternalImpulse {
                impulse: imp,
                torque_impulse: 0.0,
            },
        ));
    }
}

// the embers fade every tick of the ember timer, when they reach the end they despawn
fn despawn_particles (
    mut commands: Commands,
    mut query: Query<(Entity, &mut EmberComponent)>,
    //key_press: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut ember_timer: ResMut<EmberTimer>,
) {
    if ember_timer.0.tick(time.delta()).just_finished() {
        for (entity, mut flame) in query.iter_mut() {
            match flame.state {
                EmberStrength::Full => flame.state = EmberStrength::Strong,
                EmberStrength::Strong => flame.state = EmberStrength::Normal,
                EmberStrength::Normal => flame.state = EmberStrength::Weak,
                EmberStrength::Weak => commands.entity(entity).despawn(),
            }
        }
    }
}