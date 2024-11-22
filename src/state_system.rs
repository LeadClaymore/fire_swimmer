use bevy::prelude::*;
//use bevy_rapier2d::prelude::*;

#[derive(Bundle)]
pub struct StateBundle {
    //tbh IDK if I need this
}

pub struct StateSystemPlugin;

impl Plugin for StateSystemPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_state(AppState::LoadingScreen)
        ;
    }
}

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    LoadingScreen,
    //MainMenu,
    InGame,
}

