
use std::collections::BTreeSet;
use valence::block::*;
use valence::interact_block::*;
use valence::inventory::*;
use valence::prelude::*;
use valence::math::IVec3;
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
        let real_pos = event.position.get_in_direction(event.face);
        let Some(block) = layer.block(real_pos) else {continue;};
        if !block.state.is_replaceable() {continue;};
        println!("{:?} {:?}", event.face, event.cursor_pos);
        let mut state = block_kind.to_state();

        match event.face {
            Direction::Up | Direction::Down  => {
                let real_pos_vec3:Vec3 = (real_pos.x as f32, real_pos.y as f32, real_pos.z as f32).into();
                let real_pos_vec3 = real_pos_vec3+event.cursor_pos;
                let position_vec3:Vec3 = (position.x as f32, position.y as f32, position.z as f32).into();
                let delta:Vec3 = real_pos_vec3 - position_vec3;
                let direction_from_player = if delta.x.abs() < delta.z.abs() {
                    if delta.z < 0.0{ PropValue::North } else { PropValue::South } }
                else { if delta.x > 0.0{ PropValue::East } else { PropValue::West } }; 

                state = state.set(
                    PropName::Facing,
                    direction_from_player,
                 );
            },
            _ => {
                if let Some(block) = state.wall_block_id() {
                    println!("{:?} has a wall block", state);
                    state = block;
                    state = state.set(
                        PropName::Facing,
                        match event.face{
                            Direction::West => PropValue::West,
                            Direction::East => PropValue::East,
                            Direction::North => PropValue::North,
                            Direction::South => PropValue::South,
                            _ => unreachable!()

                        }
                     );
                }else {
                    state = state.set(
                        PropName::Facing,
                        match event.face{
                            Direction::West => PropValue::East,
                            Direction::East => PropValue::West,
                            Direction::North => PropValue::South,
                            Direction::South => PropValue::North,
                            _ => unreachable!()

                        },
                        )
                }

            }
        }
        state = state.set(
            PropName::Axis,
            match event.face {
                Direction::Down | Direction::Up => PropValue::Y,
                Direction::North | Direction::South => PropValue::Z,
                Direction::West | Direction::East => PropValue::X,
            },
        );
        layer.set_block(real_pos, state);
    }
}
