#![allow(dead_code)]

use core::sync::atomic::AtomicUsize;

use crate::common_test::*;

#[derive(Component, Default, Clone)]
struct ModuleInvokeCounter {
    pub count: u32,
}

static MODULE_DTOR_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(Component)]
struct ModuleInvoke;

impl Drop for ModuleInvoke {
    fn drop(&mut self) {
        MODULE_DTOR_COUNTER.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
    }
}

impl Module for ModuleInvoke {
    fn module(world: &World) {
        world.get::<&mut ModuleInvokeCounter>(|invoker| {
            invoker.count += 1;
        });

        world.system::<()>().run(|_| {});
    }
}

pub(crate) mod ns {
    use super::*;

    #[derive(Component)]
    pub struct NestedModule;

    impl Module for NestedModule {
        fn module(world: &World) {
            world.module::<NestedModule>("ns::NestedModule");
            world.component_named::<Velocity>("Velocity");
        }
    }

    #[derive(Component)]
    pub struct SimpleModule;

    impl Module for SimpleModule {
        fn module(world: &World) {
            world.module::<SimpleModule>("ns::SimpleModule");
            world.import::<NestedModule>();
            world.component_named::<Position>("Position");
        }
    }

    #[derive(Component)]
    pub struct NamedModule;

    impl Module for NamedModule {
        fn module(world: &World) {
            world.module::<NamedModule>("my_scope::NamedModule");
            world.component_named::<Position>("Position");
        }
    }

    #[derive(Component)]
    pub struct ImplicitModule;

    impl Module for ImplicitModule {
        fn module(world: &World) {
            world.component_named::<Position>("Position");
        }
    }

    #[derive(Component)]
    pub struct NamedModuleInRoot;

    impl Module for NamedModuleInRoot {
        fn module(world: &World) {
            world.module::<NamedModuleInRoot>("::NamedModuleInRoot");
            world.component_named::<Position>("Position");
        }
    }

    #[derive(Component)]
    pub struct ReparentModule;

    impl Module for ReparentModule {
        fn module(world: &World) {
            let m = world.module::<ReparentModule>("");
            m.child_of(world.entity_named("::parent"));

            let other = world.entity_named("::ns::ReparentModule");
            assert!(other.id() != 0);
            assert!(other != m);
        }
    }
}

#[derive(Component)]
pub struct ReparentRootModule;

impl Module for ReparentRootModule {
    fn module(world: &World) {
        world.module::<ReparentRootModule>("ns::ReparentRootModule");
    }
}

pub(crate) mod renamed_root_module {
    use super::*;

    #[derive(Component)]
    pub struct ModuleType;

    impl Module for ModuleType {
        fn module(world: &World) {
            world.module::<ModuleType>("::MyModule");
            for _ in 0..5 {
                let e = world.entity();
                assert_eq!(*e.id(), (*e.id()) as u32 as u64);
            }
        }
    }
}

pub(crate) mod ns_parent {
    use super::*;

    #[derive(Component)]
    pub struct NsType {
        x: f32,
    }

    #[derive(Component)]
    pub struct ShorterParent;

    impl Module for ShorterParent {
        fn module(world: &World) {
            world.module::<ShorterParent>("ns::ShorterParent");
            world.component::<NsType>();
        }
    }

    #[derive(Component)]
    pub struct LongerParent;

    impl Module for LongerParent {
        fn module(world: &World) {
            world.module::<LongerParent>("ns_parent_namespace::LongerParent");
            world.component::<NsType>();
        }
    }

    pub(crate) mod ns_child {
        use super::*;

        #[derive(Component)]
        pub struct Nested;

        impl Module for Nested {
            fn module(world: &World) {
                world.module::<Nested>("ns::child::Nested");
                world.component::<NsType>();
            }
        }
    }
}

#[derive(Component)]
pub struct ModuleType;

impl Module for ModuleType {
    fn module(world: &World) {
        world.module::<ModuleType>("::ModuleType");
        world.component::<Position>();
    }
}

#[test]
fn module_import() {
    let world = World::new();

    let m = world.import::<ns::SimpleModule>();
    assert!(m.id() != 0);
    assert_eq!(m.path().unwrap(), "::flecs::module_test::ns::SimpleModule");
    assert!(m.has(flecs::Module));

    let pos = world.component::<Position>();
    assert!(pos.id() != 0);
    assert_eq!(
        pos.path().unwrap(),
        "::flecs::module_test::ns::SimpleModule::Position"
    );

    let e = world.entity().add(Position::id());
    assert!(e.id() != 0);
    assert!(e.has(Position::id()));
}

#[test]
fn module_lookup_from_scope() {
    let world = World::new();

    world.import::<ns::SimpleModule>();

    let ns_entity = world.lookup("flecs::module_test::ns");
    assert!(ns_entity.id() != 0);

    let module_entity = world.lookup("flecs::module_test::ns::SimpleModule");
    assert!(module_entity.id() != 0);

    let position_entity = world.lookup("flecs::module_test::ns::SimpleModule::Position");
    assert!(position_entity.id() != 0);

    let nested_module = ns_entity.lookup("SimpleModule");
    assert!(module_entity.id() == nested_module.id());

    let module_position = module_entity.lookup("Position");
    assert!(position_entity.id() == module_position.id());

    let ns_position = ns_entity.lookup("SimpleModule::Position");
    assert!(position_entity.id() == ns_position.id());
}

#[test]
fn module_nested_module() {
    let world = World::new();
    world.import::<ns::SimpleModule>();

    let velocity = world.lookup("flecs::module_test::ns::NestedModule::Velocity");
    assert!(velocity.id() != 0);
    assert_eq!(
        velocity.path().unwrap(),
        "::flecs::module_test::ns::NestedModule::Velocity"
    );
}

#[test]
fn module_component_redefinition_outside_module() {
    let world = World::new();

    world.import::<ns::SimpleModule>();

    let pos_comp = world.lookup("::flecs::module_test::ns::SimpleModule::Position");
    assert!(pos_comp.id() != 0);

    let pos = world.component::<Position>();
    assert!(pos.id() != 0);
    assert!(pos.id() == pos_comp.id());

    let mut childof_count = 0;
    pos_comp.each_target(flecs::ChildOf, |_| {
        childof_count += 1;
    });

    assert_eq!(childof_count, 1);
}

#[test]
fn module_tag_on_namespace() {
    let world = World::new();

    let mid = world.import::<ns::NestedModule>();
    assert!(mid.has(flecs::Module));

    let nsid = world.lookup("flecs::module_test::ns");
    assert!(nsid.has(flecs::Module));
}

#[test]
fn module_dtor_on_fini() {
    {
        let world = World::new();

        world.add(ModuleInvokeCounter::id());

        world.import::<ModuleInvoke>();

        let invoke_counter = world.cloned::<&ModuleInvokeCounter>();
        assert_eq!(invoke_counter.count, 1);
        assert_eq!(
            MODULE_DTOR_COUNTER.load(core::sync::atomic::Ordering::SeqCst),
            0
        );

        let invoke_counter = world.cloned::<&ModuleInvokeCounter>();
        assert_eq!(invoke_counter.count, 1);
    }
    assert_eq!(
        MODULE_DTOR_COUNTER.load(core::sync::atomic::Ordering::SeqCst),
        1
    );
}

#[test]
fn module_register_w_root_name() {
    let world = World::new();

    let m = world.import::<ns::NamedModule>();
    let m_lookup = world.lookup("::my_scope::NamedModule");
    assert!(m.id() != 0);
    assert!(m.id() == m_lookup.id());

    assert!(world.try_lookup("::ns::NamedModule").is_none());

    let c_lookup = world.lookup("::my_scope::NamedModule::Position");
    assert!(c_lookup.id() != 0);
    assert!(c_lookup.id() == m.lookup("Position").id());
    assert!(c_lookup.id() == world.component::<Position>().id());
    assert_eq!(
        c_lookup.path().unwrap(),
        "::my_scope::NamedModule::Position"
    );
}

#[test]
fn module_implicit_module() {
    let world = World::new();

    let m = world.import::<ns::ImplicitModule>();
    let m_lookup = world.lookup("flecs::module_test::ns::ImplicitModule");
    assert!(m.id() != 0);
    assert!(m.id() == m_lookup.id());

    let p = world.component::<Position>();
    let p_lookup = world.lookup("flecs::module_test::ns::ImplicitModule::Position");
    assert!(p.id() != 0);
    assert!(p.id() == p_lookup.id());
}

#[test]
fn module_in_namespace_w_root_name() {
    let world = World::new();

    let m = world.import::<ns::NamedModuleInRoot>();
    let m_lookup = world.lookup("::NamedModuleInRoot");
    assert!(m.id() != 0);
    assert!(m.id() == m_lookup.id());
    assert_eq!(m.path().unwrap(), "::NamedModuleInRoot");

    let p = world.component::<Position>();
    let p_lookup = world.lookup("::NamedModuleInRoot::Position");
    assert!(p.id() != 0);
    assert!(p.id() == p_lookup.id());
}

#[test]
fn module_as_entity() {
    let world = World::new();

    let m = world.import::<ns::SimpleModule>();
    assert!(m.id() != 0);

    let e = world.entity_from::<ns::SimpleModule>();
    assert!(m == e);
}

#[test]
fn module_as_component() {
    let world = World::new();

    let m = world.import::<ns::SimpleModule>();
    assert!(m.id() != 0);

    let e = world.component::<ns::SimpleModule>();
    assert!(m == e);
}

#[test]
fn module_with_core_name() {
    let world = World::new();

    let m = world.import::<ModuleType>();
    assert!(m.id() != 0);
    assert_eq!(m.path().unwrap(), "::ModuleType");

    let pos = m.lookup("Position");
    assert!(pos.id() != 0);
    assert_eq!(pos.path().unwrap(), "::ModuleType::Position");
    assert!(pos == world.entity_from::<Position>());
}

#[test]
fn module_import_addons_two_worlds() {
    let a = World::new();
    let m1 = a.import::<stats::Stats>();
    let u1 = a.import::<units::Units>();

    let b = World::new();
    let m2 = b.import::<stats::Stats>();
    let u2 = b.import::<units::Units>();

    assert!(m1 == m2);
    assert!(u1 == u2);
}

#[test]
fn module_lookup_module_after_reparent() {
    let world = World::new();

    let m = world.import::<ns::NestedModule>();
    assert_eq!(m.path().unwrap(), "::flecs::module_test::ns::NestedModule");
    assert!(world.try_lookup("::flecs::module_test::ns::NestedModule") == Some(m));
    assert!(
        unsafe {
            flecs_ecs_sys::ecs_lookup(
                world.ptr_mut(),
                c"flecs.module_test.ns.NestedModule".as_ptr(),
            )
        } == m.id()
    );

    let p = world.entity_named("p");
    m.child_of(p);
    assert_eq!(m.path().unwrap(), "::p::NestedModule");
    assert!(world.try_lookup("::p::NestedModule") == Some(m));
    assert!(
        unsafe { flecs_ecs_sys::ecs_lookup(world.ptr_mut(), c"p.NestedModule".as_ptr()) } == m.id()
    );

    assert!(world.try_lookup("::ns::NestedModule").is_none());

    let e = world.entity_named("::ns::NestedModule");
    assert!(e != m);

    // Tests if symbol resolving (used by query DSL) interferes with getting the
    // correct object
    assert_eq!(
        world
            .query::<()>()
            .expr("(ChildOf, p.NestedModule)")
            .build()
            .count(),
        1
    );
    assert_eq!(
        world
            .query::<()>()
            .expr("(ChildOf, ns.NestedModule)")
            .build()
            .count(),
        0
    );
}

#[test]
fn module_reparent_module_in_ctor() {
    let world = World::new();

    let m = world.import::<ns::ReparentModule>();
    assert_eq!(m.path().unwrap(), "::parent::ReparentModule");

    let other = world.try_lookup("::ns::ReparentModule");
    assert!(other.is_some());
    assert!(other.unwrap() != m);
}

#[test]
fn module_rename_namespace_shorter() {
    let world = World::new();

    let m = world.import::<ns_parent::ShorterParent>();
    assert!(m.has(flecs::Module));
    assert_eq!(m.path().unwrap(), "::ns::ShorterParent");
    assert!(world.try_lookup("::ns_parent").is_none());
    assert!(world.try_lookup("::ns_parent::ShorterParent").is_none());
    assert!(
        world
            .try_lookup("::ns_parent::ShorterParent::NsType")
            .is_none()
    );
    assert!(world.try_lookup("::ns::ShorterParent::NsType").is_some());

    let ns = world.try_lookup("::ns");
    assert!(ns.is_some());
    assert!(ns.unwrap().has(flecs::Module));
}

#[test]
fn module_rename_namespace_longer() {
    let world = World::new();

    let m = world.import::<ns_parent::LongerParent>();
    assert!(m.has(flecs::Module));
    assert_eq!(m.path().unwrap(), "::ns_parent_namespace::LongerParent");
    assert!(world.try_lookup("::ns_parent").is_none());
    assert!(world.try_lookup("::ns_parent::LongerParent").is_none());
    assert!(
        world
            .try_lookup("::ns_parent::LongerParent::NsType")
            .is_none()
    );
    assert!(
        world
            .try_lookup("::ns_parent_namespace::LongerParent::NsType")
            .is_some()
    );

    let ns = world.try_lookup("::ns_parent_namespace");
    assert!(ns.is_some());
    assert!(ns.unwrap().has(flecs::Module));
}

#[test]
fn module_rename_namespace_nested() {
    let world = World::new();

    let m = world.import::<ns_parent::ns_child::Nested>();
    assert!(m.has(flecs::Module));
    assert_eq!(m.path().unwrap(), "::ns::child::Nested");
    assert!(world.try_lookup("::ns::child::Nested::NsType").is_some());
    assert!(
        world
            .try_lookup("::ns_parent::ns_child::Nested::NsType")
            .is_none()
    );
    assert!(world.try_lookup("::ns_parent::ns_child::Nested").is_none());
    assert!(world.try_lookup("::ns_parent::ns_child").is_none());
    assert!(world.try_lookup("::ns_parent").is_none());

    let ns = world.try_lookup("::ns");
    assert!(ns.is_some());
    assert!(ns.unwrap().has(flecs::Module));

    let ns_child = world.try_lookup("::ns::child");
    assert!(ns_child.is_some());
    assert!(ns_child.unwrap().has(flecs::Module));
}

#[test]
fn module_rename_reparent_root_module() {
    let world = World::new();

    let m = world.import::<ReparentRootModule>();
    let p = m.parent();
    assert!(p.is_some());
    let p = p.unwrap();
    assert_eq!(p.get_name().unwrap(), "ns");
    assert_eq!(m.get_name().unwrap(), "ReparentRootModule");
}

#[test]
fn module_no_recycle_after_rename_reparent() {
    let world = World::new();

    let m = world.import::<renamed_root_module::ModuleType>();
    let p = m.parent();
    assert!(p.is_none());
    assert_eq!(m.get_name().unwrap(), "MyModule");
}

#[test]
fn module_reimport_after_delete() {
    let world = World::new();

    {
        let m = world.import::<ModuleType>();
        assert!(m.lookup("Position") == world.component::<Position>());
        assert!(m == world.entity_from::<ModuleType>());
    }

    world.entity_from::<ModuleType>().destruct();

    {
        let m = world.import::<ModuleType>();
        assert!(m.lookup("Position") == world.component::<Position>());
        assert!(m == world.entity_from::<ModuleType>());
    }
}

#[derive(Component)]
struct ModuleA;

#[derive(Component)]
struct ModuleAComponent;

impl Module for ModuleA {
    fn module(world: &World) {
        world.module::<ModuleA>("::ModuleA");
        world.component::<ModuleAComponent>();
    }
}

#[test]
fn module_component_name_w_module_name() {
    let world = World::new();

    let m = world.import::<ModuleA>();
    assert!(m.id() != 0);
    let c = world.try_lookup("ModuleA::ModuleAComponent");
    assert!(c.is_some());
    let c = c.unwrap();
    assert_eq!(c.get_name().unwrap(), "ModuleAComponent");
    assert_eq!(c.parent().unwrap().get_name().unwrap(), "ModuleA");
}
