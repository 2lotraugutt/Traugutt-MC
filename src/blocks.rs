
use std::collections::BTreeSet;
use valence::block::*;
use valence::interact_block::*;
use valence::inventory::*;
use valence::prelude::*;
// use valence::*;

pub struct BlocksPlugin;
impl Plugin for BlocksPlugin {
    fn build(&self, app: &mut App) {
        let block_resource = BlocksResource{enabled_for: BTreeSet::new()};
        app
            .insert_resource(block_resource)
            // .add_systems(Startup, setup);
            // .add_command::<LoginCommand>()
            // .add_event::<LoginEvent>()
            .add_systems(Update, (digging, place_blocks));
    }
}

#[derive(Debug, Resource)]
pub struct BlocksResource {
    pub enabled_for: BTreeSet<Entity>,
}

fn digging(
    mut events: EventReader<DiggingEvent>,
    clients: Query<(
    &GameMode, 
    &VisibleChunkLayer
    )>,
    mut layers: Query<&mut ChunkLayer>,
    resource: Res<BlocksResource>,
) {
    for event in events.read() {
        let Ok((game_mode, layer)) = clients.get(event.client) else {
            continue;
        };
        if !(resource.enabled_for.contains(&layer.0)){ continue;};
        let Ok(mut layer) = layers.get_mut(layer.0) else {continue;};

        if (*game_mode == GameMode::Creative && event.state == DiggingState::Start)
            || (*game_mode == GameMode::Survival && event.state == DiggingState::Stop)
        {
            layer.set_block(event.position, BlockState::AIR);
        }
    }
}

fn place_blocks(
    mut clients: Query<(&mut Inventory, &GameMode, &HeldItem, &VisibleChunkLayer)>,
    mut layers: Query<&mut ChunkLayer>,
    mut events: EventReader<InteractBlockEvent>,
    resource: Res<BlocksResource>,
) {
    for event in events.read() {
        let Ok((mut inventory, game_mode, held, layer)) = clients.get_mut(event.client) else {
            continue;
        };
        if !(resource.enabled_for.contains(&layer.0)){ continue;};
        let Ok(mut layer) = layers.get_mut(layer.0) else {continue;};
        if event.hand != Hand::Main {
            continue;
        }

        // get the held item
        let slot_id = held.slot();
        let stack = inventory.slot(slot_id);
        if stack.is_empty() {
            // no item in the slot
            continue;
        };

        let Some(block_kind) = BlockKind::from_item_kind(stack.item) else {
            // can't place this item as a block
            continue;
        };

        if *game_mode == GameMode::Survival {
            // check if the player has the item in their inventory and remove
            // it.
            if stack.count > 1 {
                let amount = stack.count - 1;
                inventory.set_slot_amount(slot_id, amount);
            } else {
                inventory.set_slot(slot_id, ItemStack::EMPTY);
            }
        }
        let real_pos = event.position.get_in_direction(event.face);
        println!("{:?} {:?}", event.face, event.cursor_pos);
        let state = block_kind.to_state().set(
            PropName::Facing,
            match event.face {
                Direction::Down => PropValue::Down,
                Direction::Up => PropValue::Up,
                Direction::West => PropValue::East,
                Direction::East => PropValue::West,
                Direction::North => PropValue::South,
                Direction::South => PropValue::North,
            }
        );
        layer.set_block(real_pos, state);
    }
}
