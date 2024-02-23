use valence::*;
use valence::prelude::*;
use valence::command::handler::CommandResultEvent;
use valence::command_macros::Command;

pub fn worldgen(
    layer: &mut LayerBundle,
) {
    for x in -1..=0 {
        for y in -1..=0 {
            layer.chunk.insert_chunk([0, 0], UnloadedChunk::new());
        }
    }
    let block = BlockState::BARRIER;
    // let block = BlockState::STONE;
    layer.chunk.set_block([1, 127, 1], block);
    layer.chunk.set_block([1, 130, 1], block);
    for y in 128..=129 {
            layer.chunk.set_block([2, y, 1], block);
            layer.chunk.set_block([0, y, 1], block);
            layer.chunk.set_block([1, y, 2], block);
            layer.chunk.set_block([1, y, 0], block);
    }
}

#[derive(Command, Debug, Clone)]
#[paths("login {password}")]
#[scopes("logedout.login")]
#[allow(dead_code)]
pub(crate) struct LoginCommand {
    password: String
}

fn handle_struct_command(
    mut events: EventReader<CommandResultEvent<LoginCommand>>,
    mut clients: Query<&mut Client>,
) {
    for event in events.read() {
        let client = &mut clients.get_mut(event.executor).unwrap();
        client.send_chat_message(format!(
            "Struct command executed with data:\n {}", 
            &event.result.password
        ));
    }
}
