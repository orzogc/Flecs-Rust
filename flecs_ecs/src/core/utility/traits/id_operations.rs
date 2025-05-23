use crate::core::*;
use crate::sys;

pub trait IdOperations<'a>: WorldProvider<'a> + IntoId + Sized + Copy {
    type IdType;

    fn id(&self) -> Self::IdType;

    fn new_from_id(world: impl WorldProvider<'a>, id: impl IntoId) -> Self;

    fn new_from_str(world: impl WorldProvider<'a>, expr: &str) -> Self;

    /// checks if the id is a wildcard
    ///
    /// # See also
    ///
    /// * C API: `ecs_id_is_wildcard`
    fn is_wildcard(self) -> bool {
        let world = self.world();
        unsafe { sys::ecs_id_is_wildcard(*self.into_id(world)) }
    }

    /// Return id with role added
    #[inline(always)]
    fn add_flags(self, flags: impl IntoId) -> Self {
        let world = self.world();
        Self::new_from_id(self.world(), self.into_id(world) | flags.into_id(world))
    }

    /// Return id with role removed.
    /// This function checks if the id has the specified role, and if it does not, the function will assert.
    #[inline(always)]
    fn remove_flags_checked(self, _flags: impl IntoId) -> Self {
        let world = self.world();
        ecs_assert!(
            self.into_id(world) & RUST_ecs_id_FLAGS_MASK == _flags.into_id(world),
            FlecsErrorCode::InvalidParameter
        );

        Self::new_from_id(self.world(), self.into_id(world) & RUST_ECS_COMPONENT_MASK)
    }

    /// Return id with role removed
    #[inline(always)]
    fn remove_flags(self) -> Self {
        Self::new_from_id(
            self.world(),
            self.into_id(self.world()) & RUST_ECS_COMPONENT_MASK,
        )
    }

    /// Get flags associated with id
    ///
    /// # Returns
    ///
    /// The flags associated with the id or 0 Entity if the id is not in use
    #[inline(always)]
    fn flags(self) -> Self {
        let world = self.world();
        Self::new_from_id(world, self.into_id(world) & RUST_ecs_id_FLAGS_MASK)
    }

    /// Test if id has specified role
    #[inline(always)]
    fn has_flags_for(self, flags: u64) -> bool {
        self.into_id(self.world()) & flags == flags
    }

    /// Test if id has any role
    #[inline(always)]
    fn has_any_flags(self) -> bool {
        self.into_id(self.world()) & RUST_ecs_id_FLAGS_MASK != 0
    }

    /// Return id without role
    #[inline(always)]
    fn remove_generation(self) -> EntityView<'a> {
        EntityView::new_from(self.world(), *self.into_id(self.world()) as u32 as u64)
    }

    /// Convert id to string
    ///
    /// # See also
    ///
    /// * C API: `ecs_id_str`
    #[inline(always)]
    fn to_str(self) -> &'a str {
        let world = self.world();
        // SAFETY: We assume that `ecs_id_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_id_ptr never returns null, so we don't need to check for that.
        if let Ok(str) = unsafe {
            core::ffi::CStr::from_ptr(sys::ecs_id_str(self.world_ptr(), *self.into_id(world)))
        }
        .to_str()
        {
            str
        } else {
            ecs_assert!(
                false,
                FlecsErrorCode::UnwrapFailed,
                "Failed to convert id to string (id: {})",
                self.into_id(world)
            );

            "invalid_str_from_id"
        }
    }

    /// Convert id to string
    ///
    /// # Safety
    /// safe version : '`str`'
    /// This function is unsafe because it assumes that the id is valid.
    ///
    /// # See also
    ///
    /// * C API: `ecs_id_str`
    #[inline(always)]
    unsafe fn to_str_unchecked(self) -> &'a str {
        // SAFETY: We assume that `ecs_id_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_id_ptr never returns null, so we don't need to check for that.
        let c_str_ptr = unsafe { sys::ecs_id_str(self.world_ptr(), *self.into_id(self.world())) };

        // SAFETY: We assume the C string is valid UTF-8. This is risky if not certain.
        unsafe { core::str::from_utf8_unchecked(core::ffi::CStr::from_ptr(c_str_ptr).to_bytes()) }
    }

    /// Convert role of id to string.
    ///
    /// # See also
    ///
    /// * C API: `ecs_id_flag_str`
    #[inline(always)]
    fn flags_str(self) -> &'a str {
        let world = self.world();
        // SAFETY: We assume that `ecs_role_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_role_str never returns null, so we don't need to check for that.
        unsafe {
            core::ffi::CStr::from_ptr(sys::ecs_id_flag_str(
                *self.into_id(world) & RUST_ecs_id_FLAGS_MASK,
            ))
        }
        .to_str()
        .unwrap_or({
            ecs_assert!(
                false,
                FlecsErrorCode::UnwrapFailed,
                "Failed to convert id to string (id: {})",
                self.into_id(world)
            );
            "invalid_str_from_id"
        })
    }

    /// Convert role of id to string.
    /// # Safety
    /// safe version : '`to_flags_str`'
    /// This function is unsafe because it assumes that the id is valid.
    ///
    /// # See also
    ///
    /// * C API: `ecs_id_flag_str`
    #[inline(always)]
    unsafe fn to_flags_str_unchecked(self) -> &'a str {
        // SAFETY: We assume that `ecs_id_str` returns a pointer to a null-terminated
        // C string with a static lifetime. The caller must ensure this invariant.
        // ecs_id_ptr never returns null, so we don't need to check for that.
        let c_str_ptr =
            unsafe { sys::ecs_id_flag_str(*self.into_id(self.world()) & RUST_ecs_id_FLAGS_MASK) };

        // SAFETY: We assume the C string is valid UTF-8. This is risky if not certain.
        unsafe { core::str::from_utf8_unchecked(core::ffi::CStr::from_ptr(c_str_ptr).to_bytes()) }
    }
}
