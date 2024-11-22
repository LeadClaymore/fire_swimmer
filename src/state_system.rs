use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Bundle)]
pub struct StateBundle {
    //tbh IDK if I need this
}

pub struct State_System_Plugin;

impl Plugin for State_System_Plugin {
    fn build(&self, app: &mut App) {
        app
            .insert_state(AppState::LoadingScreen)
        ;
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
enum AppState {
    LoadingScreen,
    MainMenu,
    InGame,
}

