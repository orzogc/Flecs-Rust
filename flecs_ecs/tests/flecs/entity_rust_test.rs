#![allow(dead_code)]

use crate::common_test::*;

#[test]
fn count_target_ids() {
    let world = World::new();

    let e = world.entity();
    let r = world.entity();
    let o = world.entity();

    e.add((r, o));
    e.add((r, o));

    assert_eq!(e.target_id_count(r).unwrap(), 1);

    let e2 = world.entity();
    e2.add((r, o));

    assert_eq!(e.target_id_count(r).unwrap(), 1);
    assert_eq!(e2.target_id_count(r).unwrap(), 1);

    let o2 = world.entity();

    e.add((r, o2));

    assert_eq!(e.target_id_count(r).unwrap(), 2);
    assert_eq!(e2.target_id_count(r).unwrap(), 1);
}

#[test]
fn entity_id_reuse() {
    let world = World::new();

    let a = world.entity_named("a");
    let b = world.entity().child_of(a);
    let first_archetype = b.archetype().to_string();
    a.destruct();

    let a = world.entity_named("a");
    let b = world.entity().child_of(a);
    assert!(
        b.id() > u32::MAX as u64,
        "this test is not valid if the id was not reused"
    );
    assert_eq!(b.archetype().to_string(), first_archetype);
}
