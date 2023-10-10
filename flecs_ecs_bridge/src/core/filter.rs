use super::{
    c_binding::bindings::{
        _ecs_abort, ecs_filter_copy, ecs_filter_desc_t, ecs_filter_fini, ecs_filter_init,
        ecs_filter_iter, ecs_filter_move, ecs_filter_next, ecs_filter_str, ecs_get_entity,
        ecs_os_api, ECS_FILTER_INIT,
    },
    c_types::{FilterT, TermT, WorldT},
    entity::Entity,
    iterable::Iterable,
    term::Term,
    utility::errors::FlecsErrorCode,
};
use std::ffi::c_char;
use std::ops::Deref;

pub struct FilterBase {
    pub world: *mut WorldT,
    pub filter_ptr: *mut FilterT,
}

impl Default for FilterBase {
    fn default() -> Self {
        FilterBase {
            world: std::ptr::null_mut(),
            filter_ptr: std::ptr::null_mut(),
        }
    }
}
impl FilterBase {
    pub fn each<T, F>(&self, mut func: F)
    where
        T: Iterable<F>,
        F: FnMut(Entity, T),
    {
        unsafe {
            let mut iter = ecs_filter_iter(self.world, self.filter_ptr);
            let mut func_ref = &mut func;
            while ecs_filter_next(&mut iter) {
                for i in 0..iter.count as usize {
                    let entity =
                        Entity::new_from_existing(self.world, *iter.entities.add(i as usize));
                    let tuple = T::get_data(&iter, i);
                    tuple.apply(entity, &mut func_ref);
                }
            }
        }
    }

    pub fn new(world: *mut WorldT, filter: *mut FilterT) -> Self {
        FilterBase {
            world,
            filter_ptr: filter,
        }
    }

    pub fn new_ownership(world: *mut WorldT, filter: *mut FilterT) -> Self {
        let filter_obj = FilterBase {
            world,
            filter_ptr: std::ptr::null_mut(),
        };

        unsafe { ecs_filter_move(filter_obj.filter_ptr, filter as *mut FilterT) };

        filter_obj
    }

    pub fn new_from_desc(world: *mut WorldT, desc: *mut ecs_filter_desc_t) -> Self {
        let filter_obj = FilterBase {
            world,
            filter_ptr: std::ptr::null_mut(),
        };

        todo!("this seems wrong");
        unsafe {
            (*desc).storage = filter_obj.filter_ptr;
        }

        unsafe {
            if ecs_filter_init(filter_obj.world, desc) == std::ptr::null_mut() {
                _ecs_abort(
                    FlecsErrorCode::InvalidParameter.to_int(),
                    file!().as_ptr() as *const i8,
                    line!() as i32,
                    std::ptr::null(),
                );

                if let Some(abort_func) = ecs_os_api.abort_ {
                    abort_func()
                }
            }

            if !(*desc).terms_buffer.is_null() {
                if let Some(free_func) = ecs_os_api.free_ {
                    free_func((*desc).terms_buffer as *mut _)
                }
            }
        }

        filter_obj
    }

    pub fn entity(&self) -> Entity {
        Entity::new_from_existing(self.world, unsafe {
            ecs_get_entity(self.filter_ptr as *const _)
        })
    }

    pub fn each_term<F>(&self, mut func: F)
    where
        F: FnMut(Term),
    {
        unsafe {
            for i in 0..(*self.filter_ptr).term_count {
                let term = Term::new(self.world, *(*self.filter_ptr).terms.add(i as usize));
                func(term);
            }
        }
    }

    pub fn get_term(&self, index: usize) -> Term {
        Term::new(self.world, unsafe {
            *(*self.filter_ptr).terms.add(index as usize)
        })
    }

    pub fn field_count(&self) -> i32 {
        unsafe { (*self.filter_ptr).field_count }
    }

    pub fn to_string(&self) -> String {
        let result: *mut c_char =
            unsafe { ecs_filter_str(self.world, self.filter_ptr as *const _) };
        let rust_string =
            String::from(unsafe { std::ffi::CStr::from_ptr(result).to_str().unwrap() });
        unsafe {
            if let Some(free_func) = ecs_os_api.free_ {
                free_func(result as *mut _)
            }
        }
        rust_string
    }
}

impl Drop for FilterBase {
    fn drop(&mut self) {
        if !self.filter_ptr.is_null() {
            unsafe { ecs_filter_fini(&mut self.filter_ptr as *const _ as *mut _) }
        }
    }
}

impl Clone for FilterBase {
    fn clone(&self) -> Self {
        let mut new_filter = FilterBase::default();
        new_filter.world = self.world;
        if !self.filter_ptr.is_null() {
            new_filter.filter_ptr = self.filter_ptr.clone();
        } else {
            new_filter.filter_ptr = std::ptr::null_mut();
        }
        unsafe { ecs_filter_copy(new_filter.filter_ptr, self.filter_ptr) };
        new_filter
    }
}

///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////

pub struct Filter<T, F>
where
    T: Iterable<F>,
    F: FnMut(Entity, T),
{
    pub filter: FilterBase,
    pub desc: ecs_filter_desc_t,
    next_term_index: usize,
    _phantom: std::marker::PhantomData<T>,
    _phantom2: std::marker::PhantomData<F>,
}

impl<T, F> Deref for Filter<T, F>
where
    T: Iterable<F>,
    F: FnMut(Entity, T),
{
    type Target = FilterBase;

    fn deref(&self) -> &Self::Target {
        &self.filter
    }
}
impl<T, F> Filter<T, F>
where
    T: Iterable<F>,
    F: FnMut(Entity, T),
{
    pub fn new(world: *mut WorldT) -> Self {
        unsafe {
            let mut desc = ecs_filter_desc_t::default();
            T::register_ids_descriptor(world, &mut desc);
            let raw_filter = unsafe { ecs_filter_init(world, &desc) };
            let filter = Filter {
                filter: FilterBase {
                    world,
                    filter_ptr: raw_filter,
                },
                desc,
                next_term_index: 0,
                _phantom: std::marker::PhantomData,
                _phantom2: std::marker::PhantomData,
            };
            filter
        }
        //T::populate(&mut filter);
    }

    pub fn current_term(&mut self) -> &mut TermT {
        &mut self.desc.terms[self.next_term_index]
    }

    pub fn next_term(&mut self) {
        self.next_term_index += 1;
    }
}
