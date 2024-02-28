use std::env;
// use std::io;
// use std::io::Write;

use valence::*;
use valence::prelude::*;
// use valence::anvil::*;

use crate::login::LoginEvent;
// use std::path::PathBuf;
use valence::anvil::parsing::*;

// use valence::command::handler::CommandResultEvent;
// use valence::command::scopes::CommandScopes;
// use valence::command_macros::Command;

#[derive(Debug, Resource)]
pub struct SpawnResource {
    pub layer_id: Option<Entity>,
}
impl SpawnResource {
    pub fn new() -> SpawnResource{
        SpawnResource{layer_id: None}
    }
}

pub struct SpawnPlugin;
impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        let login_asset = SpawnResource::new();
        app
            .insert_resource(login_asset)
            .add_systems(Startup, setup)
            .add_systems(Update, handle_login_event);
            // .add_systems(Update, handle_chunk_loads);
    }
}

static CHUNKS: i32 = 16;
// fn load_chunk_line(path: impl Into<PathBuf>, biomes: &BiomeRegistry, z: i32) -> Vec<(ChunkPos,UnloadedChunk)> {
// let mut ret: Vec<(ChunkPos,UnloadedChunk)> = vec![];
//     let mut anvil_dimention_folder = DimensionFolder::new(path, &biomes);
//     for x in -CHUNKS/2..CHUNKS/2 {
//         let pos = ChunkPos::new(x, z);
//         let chunk = anvil_dimention_folder.get_chunk(pos).unwrap().unwrap().chunk ;
//         ret.push((pos, chunk));
//         // level.ignored_chunks.insert(pos);
//         // level.force_chunk_load(pos);
//     }
//     ret
// }

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
    mut spawn_resource: ResMut<SpawnResource>
) {
    let mut layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);
    let spawn_world = env::var("SPAWN_WORLD").unwrap_or("spawn".to_string());
    // let mut level = AnvilLevel::new(spawn_world, &biomes);
    let mut anvil_dimention_folder = DimensionFolder::new(spawn_world, &biomes);
    // let mut stdout_f = io::stdout();
    for z in -CHUNKS/2..CHUNKS/2 {
        // print!("\r\x1b[33mLoading Spawn: \x1b[33;1m{}%\x1b[0m", 100*(z+CHUNKS/2)/CHUNKS);
        // let _ =stdout_f.flush();
        for x in -CHUNKS/2..CHUNKS/2 {
            let pos = ChunkPos::new(x, z);
            let chunk = anvil_dimention_folder.get_chunk(pos).unwrap().unwrap().chunk ;
            layer.chunk.insert_chunk(pos, chunk);
        }
    }

    println!("\x1b[32;1m\rLoading Spawn Complete \x1b[0m(Loaded {} Chunks)\x1b[0m", layer.chunk.chunks().count());
    // // worldgen(&mut layer);

    let spawned = commands.spawn(layer);
    spawn_resource.layer_id = Some(spawned.id());
}

const SPAWN_POS: [f64; 3] = [
    0 as f64,
    0 as f64,
    0 as f64,
];
fn handle_login_event(
    mut events: EventReader<LoginEvent>,
    spawn_resource: ResMut<SpawnResource>,
    mut clients: Query<(
            &mut Client, // client
            &mut EntityLayerId, // layer_id
            &mut VisibleChunkLayer, // visable_chunk_layer
            &mut VisibleEntityLayers, // visable_chunk_layer
            &mut Position,
            &mut GameMode, // game_mode
        )>,
){
    for event in events.read() {
        let (
            ref mut client,
            ref mut layer_id,
            ref mut visible_chunk_layer,
            ref mut visible_entity_layer,
            ref mut pos,
            ref mut gamemode,
        ) = &mut clients.get_mut(event.player).unwrap();
        layer_id.0 = spawn_resource.layer_id.unwrap();
        visible_chunk_layer.0 = spawn_resource.layer_id.unwrap();
        visible_entity_layer.0.insert(spawn_resource.layer_id.unwrap());
        pos.set(SPAWN_POS);
        **gamemode = GameMode::Creative;
        client.send_chat_message(
            "This is the waiting room. ".into_text() +
            "Wait here".into_text().color(Color::BLUE).bold() +
            " to the begining of the competition".into_text()
        );
    }

}
