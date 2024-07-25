use flecs_ecs::prelude::*;

use super::Opaque;

//impl_component_traits_binding_type!(EcsMember);
//impl_component_traits_binding_type!(EcsEnumConstant);
//impl_component_traits_binding_type!(EcsBitmaskConstant);

impl<T: ComponentId> FlecsConstantId for Opaque<'static, T> {
    const ID: u64 = ECS_OPAQUE;
}

impl<T: ComponentId> DataComponent for Opaque<'static, T> {}

impl<T: ComponentId> ComponentType<flecs_ecs::core::Struct> for Opaque<'static, T> {}

impl<T: ComponentId> ComponentInfo for Opaque<'static, T> {
    const IS_ENUM: bool = false;
    const IS_TAG: bool = false;
    type TagType = FlecsFirstIsNotATag;
    const IMPLS_CLONE: bool = true;
    const IMPLS_DEFAULT: bool = true;
    const IS_REF: bool = false;
    const IS_MUT: bool = false;

    const IS_GENERIC: bool = true;
}

impl<T: ComponentId> ComponentId for Opaque<'static, T> {
    type UnderlyingType = Opaque<'static, T>;
    type UnderlyingEnumType = NoneEnum;

    fn __register_lifecycle_hooks(type_hooks: &mut TypeHooksT) {
        register_lifecycle_actions::<Opaque<'static, T>>(type_hooks);
    }
    fn __register_default_hooks(_type_hooks: &mut TypeHooksT) {}

    fn __register_clone_hooks(_type_hooks: &mut TypeHooksT) {}

    #[inline(always)]
    fn index() -> u32 {
        static INDEX: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(u32::MAX);
        Self::get_or_init_index(&INDEX)
    }

    fn __register_or_get_id<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl IntoWorld<'a>,
    ) -> EntityT {
        ECS_OPAQUE
    }

    fn __register_or_get_id_named<'a, const MANUAL_REGISTRATION_CHECK: bool>(
        _world: impl IntoWorld<'a>,
        _name: &str,
    ) -> EntityT {
        ECS_OPAQUE
    }

    fn is_registered_with_world<'a>(_: impl IntoWorld<'a>) -> bool {
        true
    }

    fn id<'a>(_world: impl IntoWorld<'a>) -> IdT {
        ECS_OPAQUE
    }
}
