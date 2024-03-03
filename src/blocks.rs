
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
    mut clients: Query<(&mut Inventory, &GameMode, &HeldItem, &VisibleChunkLayer, &Position)>,
    mut layers: Query<&mut ChunkLayer>,
    mut events: EventReader<InteractBlockEvent>,
    resource: Res<BlocksResource>,
) {
    for event in events.read() {
        let Ok((mut inventory, game_mode, held, layer,position)) = clients.get_mut(event.client) else {
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
            continue
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
        let Some(block) = layer.block(event.position) else {continue;};
        let block = block.state;
        let real_pos = match block.is_replaceable() {
            false => event.position.get_in_direction(event.face),
            true => event.position,
        };
        let Some(block) = layer.block(real_pos) else {continue;};
        if !block.state.is_replaceable() {continue;};
        let mut state = block_kind.to_state();

        // Half
        state = state.set(
            PropName::Half,
            match event.face {
                Direction::West| Direction::East | Direction::South | Direction::North =>
                        if event.cursor_pos.y > 0.5 { PropValue::Top } else { PropValue::Bottom },
                Direction::Down => PropValue::Top,
                Direction::Up => PropValue::Bottom,
        });

        // Facing
        let delta = Vec3::new(event.position.x as f32, event.position.y as f32, event.position.z as f32) + event.cursor_pos -
        Vec3::new(position.x as f32, position.y as f32, position.z as f32);
        let direction_from_player = if delta.x.abs() < delta.z.abs() {
        if delta.z < 0.0{ Direction::North } else { Direction::South } }
        else { if delta.x > 0.0{ Direction::East } else { Direction::West } }; 
        match event.face {
                Direction::West| Direction::East | Direction::South | Direction::North 
                    => state = state.wall_block_id().unwrap_or(state),
                _ => (),
        };
        let facing = match direction_from_player.rotate(state.get_rotation_inversion()) {
            Direction::West => PropValue::West,
            Direction::East => PropValue::East,
            Direction::North => PropValue::North,
            Direction::South => PropValue::South,
            _ => unreachable!(),
        };
        state = state.set(
                PropName::Facing,
                facing
            );
        // Axis
        state = state.set(
            PropName::Axis,
            match event.face {
                Direction::Down | Direction::Up => PropValue::Y,
                Direction::North | Direction::South => PropValue::Z,
                Direction::West | Direction::East => PropValue::X,
            },
        );
        // Disalllow placing walled blocks if the wall is not solid
        // TODO check if the wall is solid (not air)
        if state.is_walled() {
            let Some(base_block) = layer.block(real_pos.get_in_direction(direction_from_player)) else {continue;};
            let base_block = base_block.state;
            if base_block.is_air() {
                continue;
            }
        }
        layer.set_block(real_pos, state);
    }
}
