use bevy::prelude::*;
//use bevy_rapier2d::prelude::*;

use crate::state_system::AppState;

#[derive(Bundle)]
pub struct LoaderBundle {
    //tbh IDK if I need this
}

pub struct AssetLoaderPlugin;

impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SceneAsset>()
            .init_resource::<LoadingAssets>()
            .add_systems(
                Update, 
                (load_assets).run_if(in_state(AppState::LoadingScreen))
            )
            .add_systems(PreStartup, preload_textures)
        ;
    }
}

//I want to store relevent data within a struct elsewhere, however I want to have handles to be sent there here
#[derive(Resource, Debug, Default)]
struct LoadingAssets {
    image_handles: Vec<Handle<Image>>,
}

#[derive(Resource, Debug, Default)]
pub struct SceneAsset {
    pub t_temp: Handle<Image>,
    pub t_scorch: Handle<Image>,
    pub t_block: Handle<Image>,
    pub t_ember: Handle<Image>,
    pub t_enemy: Handle<Image>,
    pub t_enemy_p: Handle<Image>,
    pub t_enemy2: Handle<Image>,
    pub t_enemy3: Handle<Image>,
    pub t_block_unburnable: Handle<Image>,
    pub t_block_insta_burn: Handle<Image>,
}

fn load_assets(
    mut commands: Commands,
    mut scene_assets: ResMut<SceneAsset>,
    loading_assets: Res<LoadingAssets>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    use bevy::asset::LoadState;

    let mut all_loaded = true;

    for handle in &loading_assets.image_handles {
        match asset_server.get_load_state(handle) {
            Some(LoadState::Loaded) => continue, // This asset is loaded; check the next one
            Some(LoadState::Failed(_)) => {
                eprintln!("Failed to load an asset: {:?}", handle);
                all_loaded = false;
                // Exit early since at least one asset failed to load
                break;
            }
            _ => {
                // Asset is still loading
                all_loaded = false;
                break;
            }
        }
    }

    // when all assets are done loading this will trigger. rn it runs every update
    //TODO change state opon loading all assets
    if all_loaded {
        println!("All assets loaded!");

        // I dislike just doing each asset like this from the handles,
        //TODO make the handle system more scalable and less hard coded
        scene_assets.t_temp = loading_assets.image_handles[0].clone();
        scene_assets.t_scorch = loading_assets.image_handles[1].clone();
        scene_assets.t_ember = loading_assets.image_handles[2].clone();
        scene_assets.t_block = loading_assets.image_handles[3].clone();
        scene_assets.t_block_unburnable = loading_assets.image_handles[4].clone();
        scene_assets.t_block_insta_burn = loading_assets.image_handles[5].clone();
        scene_assets.t_enemy = loading_assets.image_handles[6].clone();
        scene_assets.t_enemy_p = loading_assets.image_handles[7].clone();
        scene_assets.t_enemy2 = loading_assets.image_handles[8].clone();
        scene_assets.t_enemy3 = loading_assets.image_handles[9].clone();
        //scene_assets.t_ = loading_assets.image_handles[7].clone();
        
        // the loading assets is now redundent and less organgized compared to the scene assets
        commands.remove_resource::<LoadingAssets>();

        //transition the state for next frame
        next_state.set(AppState::InGame);
    }
}

fn preload_textures(
    mut handle_res: ResMut<LoadingAssets>, 
    asset_server: Res<AssetServer>,
) {
    let texture_handles: Vec<Handle<Image>> = vec![
        asset_server.load("sprites/t_temp.png"),
        asset_server.load("sprites/t_scorch.png"),
        asset_server.load("sprites/t_ember.png"),
        asset_server.load("sprites/t_wood_brown.png"),
        asset_server.load("sprites/t_slate.png"),
        asset_server.load("sprites/t_paper.png"),
        asset_server.load("sprites/t_enemy.png"),
        asset_server.load("sprites/t_enemy_p.png"),
        asset_server.load("sprites/t_enemy2.png"),
        asset_server.load("sprites/t_enemy3.png"),
        //asset_server.load("sprites/t_.png"),
    ];
    handle_res.image_handles = texture_handles;
}