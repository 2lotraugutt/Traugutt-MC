use valence::*;
use valence::prelude::*;

mod login;
mod spawn;
mod admin;
mod blocks;
mod open_world;

/// Launches the server and adds nesesary plugins
pub fn main () {
    println!("\x1b[32;1mII LO Traugutt Minecraft Torunament Server\x1b[0m");
    App::new()
        .insert_resource(NetworkSettings {
            connection_mode: ConnectionMode::Offline,
            max_players: 1024,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_systems(Update, despawn_disconnected_clients)
        .add_plugins(login::LoginPlugin) 
        .add_plugins(spawn::SpawnPlugin)
        .add_plugins(blocks::BlocksPlugin)
        .add_plugins(open_world::OpenWorldPlugin) 
        .add_plugins(admin::AdminPlugin)
        .run();
}
