use bevy::prelude::*;
//use bevy_rapier2d::prelude::*;

use crate::{state_system::AppState, Scorch};

pub struct CameraPlugin;

#[derive(Bundle)]
pub struct CameraBundle {
    pub camera_info: MainCamera,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // graphical and underlying stuff
        app
            .add_systems(OnEnter(AppState::InGame), start_camera)
            .add_systems(
                Update, 
                (camera_control).run_if(in_state(AppState::InGame))
            )
        ;
    }
}

#[derive(Component)]
pub struct MainCamera;

fn start_camera(mut commands: Commands) {
    // this is the default camera
    commands.spawn((
        // Camera2dBundle::default(),
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