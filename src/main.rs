use valence::*;
use valence::prelude::*;

mod login;
mod spawn;

pub fn main () {
    println!("\x1b[32;1mII LO Traugutt Minecraft Server\x1b[0m");
    App::new()
        .insert_resource(NetworkSettings {
            connection_mode: ConnectionMode::Offline,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugins(login::LoginPlugin) 
        .add_plugins(spawn::SpawnPlugin)
        .run();
}
