#![allow(non_camel_case_types)]

mod common;
use common::*;

// A group iterator iterates over a single group of a grouped query (see the
// group_by example for more details). This can be useful when an application
// may need to match different entities based on the context of the game, such
// as editor mode, day/night, inside/outside or location in the world.
//
// One example is that of an open game which is divided up into world
// cells. Even though a world may contain many entities, only the entities in
// cells close to the player need to be processed.
//
// Instead of creating a cached query per world cell, which could be expensive
// as there are more caches to keep in sync, applications can create a single
// query grouped by world cell, and use group iterators to only iterate the
// necessary cells.

// A world cell relationship with four cells

#[derive(Debug, Default, Clone, Component)]
struct WorldCell;

#[derive(Debug, Default, Clone, Component)]
struct Cell_0_0;

#[derive(Debug, Default, Clone, Component)]
struct Cell_0_1;

#[derive(Debug, Default, Clone, Component)]
struct Cell_1_0;

#[derive(Debug, Default, Clone, Component)]
struct Cell_1_1;

// Npc tags
#[derive(Debug, Default, Clone, Component)]
struct Npc;

#[derive(Debug, Default, Clone, Component)]
struct Merchant;

#[derive(Debug, Default, Clone, Component)]
struct Soldier;

#[derive(Debug, Default, Clone, Component)]
struct Beggar;

#[derive(Debug, Default, Clone, Component)]
struct Mage;

fn main() {
    let world = World::new();

    // Create npc's in world cell 0_0
    world
        .new_entity()
        .add_pair::<WorldCell, Cell_0_0>()
        .add::<Merchant>()
        .add::<Npc>();
    world
        .new_entity()
        .add_pair::<WorldCell, Cell_0_0>()
        .add::<Merchant>()
        .add::<Npc>();

    // Create npc's in world cell 0_1
    world
        .new_entity()
        .add_pair::<WorldCell, Cell_0_1>()
        .add::<Beggar>()
        .add::<Npc>();
    world
        .new_entity()
        .add_pair::<WorldCell, Cell_0_1>()
        .add::<Soldier>()
        .add::<Npc>();

    // Create npc's in world cell 1_0
    world
        .new_entity()
        .add_pair::<WorldCell, Cell_1_0>()
        .add::<Mage>()
        .add::<Npc>();
    world
        .new_entity()
        .add_pair::<WorldCell, Cell_1_0>()
        .add::<Beggar>()
        .add::<Npc>();

    // Create npc's in world cell 1_1
    world
        .new_entity()
        .add_pair::<WorldCell, Cell_1_1>()
        .add::<Soldier>()
        .add::<Npc>();

    let query = world
        .query_builder::<(&Npc,)>()
        .group_by::<WorldCell>()
        .build();

    // Iterate all tables
    println!("All tables");

    query.iter_only(|iter| {
        let group: Entity = world.new_entity_w_id(iter.get_group_id());
        println!(
            "group: {:?} - Table [{}]",
            group.get_hierarchy_path().unwrap(),
            iter.get_table().to_string().unwrap()
        );
    });

    println!();

    println!("Tables for cell 1_0:");

    // Output:
    //  All tables
    //  group: "::Cell_0_0" - Table [Merchant, Npc, (WorldCell,Cell_0_0)]
    //  group: "::Cell_0_1" - Table [Npc, Beggar, (WorldCell,Cell_0_1)]
    //  group: "::Cell_0_1" - Table [Npc, Soldier, (WorldCell,Cell_0_1)]
    //  group: "::Cell_1_0" - Table [Npc, Mage, (WorldCell,Cell_1_0)]
    //  group: "::Cell_1_0" - Table [Npc, Beggar, (WorldCell,Cell_1_0)]
    //  group: "::Cell_1_1" - Table [Npc, Soldier, (WorldCell,Cell_1_1)]

    //todo!("iter_iterable class not implemented yet.")
}