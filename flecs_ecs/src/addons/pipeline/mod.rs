//! Pipelines order and schedule systems for execution.

mod pipeline_builder;
pub use pipeline_builder::*;

use core::ops::{Deref, DerefMut};

use crate::core::*;
use crate::sys;

/// Pipelines order and schedule systems for execution.
///
/// These are typically constructed via [`World::pipeline()`].
pub struct Pipeline<'a, T>
where
    T: QueryTuple,
{
    entity: EntityView<'a>,
    phantom: core::marker::PhantomData<T>,
}

impl<'a, T> Deref for Pipeline<'a, T>
where
    T: QueryTuple,
{
    type Target = EntityView<'a>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.entity
    }
}

impl<T> DerefMut for Pipeline<'_, T>
where
    T: QueryTuple,
{
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entity
    }
}

impl<'a, T> From<Pipeline<'a, T>> for EntityView<'a>
where
    T: QueryTuple,
{
    fn from(pipeline: Pipeline<'a, T>) -> Self {
        pipeline.entity
    }
}

impl<'a, T> From<Pipeline<'a, T>> for Entity
where
    T: QueryTuple,
{
    fn from(pipeline: Pipeline<'a, T>) -> Self {
        pipeline.id
    }
}

impl<'a, T> Pipeline<'a, T>
where
    T: QueryTuple,
{
    /// Create a new pipeline.
    ///
    /// # Arguments
    ///
    /// * `world` - The world to create the pipeline in.
    /// * `desc` - The pipeline description.
    pub(crate) fn new(world: impl WorldProvider<'a>, desc: sys::ecs_pipeline_desc_t) -> Self {
        let entity = EntityView::new(world.world());
        let mut pipeline = Self {
            entity,
            phantom: Default::default(),
        };
        pipeline.id = Entity(unsafe { sys::ecs_pipeline_init(world.world_ptr_mut(), &desc) });

        if pipeline.id == 0 {
            ecs_abort!(FlecsErrorCode::InvalidParameter);
        }

        pipeline
    }

    pub fn entity(&self) -> EntityView<'a> {
        self.entity
    }
}
