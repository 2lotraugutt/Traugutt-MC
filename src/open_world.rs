
use std::env;
use std::io;
use std::io::Write;
use crate::blocks::BlocksResource;

use valence::*;
use valence::prelude::*;
use valence::anvil::parsing::*;
// use valence::anvil::*;

pub struct OpenWorldPlugin;
impl Plugin for OpenWorldPlugin {
    fn build(&self, app: &mut App) {
        let open_world_asset = OpenWorldResource{layer_id: None};
        app
            .insert_resource(open_world_asset)
            .add_systems(Startup, setup);
            // .add_command::<LoginCommand>()
            // .add_event::<LoginEvent>()
            // .add_systems(Update, (handle_login_command,init_clients));
    }
}

#[derive(Debug, Resource)]
pub struct OpenWorldResource {
    pub layer_id: Option<Entity>,
}

const CHUNKS: i32 = 256;

fn setup(
    mut commands: Commands,
    server: Res<Server>,
    dimensions: Res<DimensionTypeRegistry>,
    biomes: Res<BiomeRegistry>,
    mut blocks_resource: ResMut<BlocksResource>,
    mut open_world_resource: ResMut<OpenWorldResource>
) {
    let mut layer = LayerBundle::new(ident!("overworld"), &dimensions, &biomes, &server);
    let open_world = env::var("OPEN_WORLD").unwrap_or("open-world".to_string());
    // let mut level = AnvilLevel::new(spawn_world, &biomes);
    let mut anvil_dimention_folder = DimensionFolder::new(open_world, &biomes);
    let mut stdout_f = io::stdout();
    for z in -CHUNKS/2..CHUNKS/2 {
        print!("\r\x1b[33mLoading World: \x1b[33;1m{}%\x1b[0m", 100*(z+CHUNKS/2)/CHUNKS);
        let _ =stdout_f.flush();
        for x in -CHUNKS/2..CHUNKS/2 {
            let pos = ChunkPos::new(x, z);
            let chunk = anvil_dimention_folder.get_chunk(pos).unwrap().unwrap().chunk ;
            layer.chunk.insert_chunk(pos, chunk);
        }
    }
    println!("\x1b[32;1m\rLoading World Complete \x1b[0m(Loaded {} Chunks)\x1b[0m", layer.chunk.chunks().count());
    // // worldgen(&mut layer);

    let spawned = commands.spawn(layer);
    blocks_resource.enabled_for.insert(spawned.id());
    open_world_resource.layer_id = Some(spawned.id());
}


