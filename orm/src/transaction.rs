use crate::{
    data::ObjectId,
    error::*,
    object::{Object, Store},
    storage::StorageTransaction,
};
use std::{
    any::{Any, TypeId},
    cell::{Cell, Ref, RefCell, RefMut},
    collections::HashMap,
    marker::PhantomData,
    rc::Rc,
};

////////////////////////////////////////////////////////////////////////////////
pub struct Transaction<'a> {
    inner: Box<dyn StorageTransaction + 'a>,
    cache: RefCell<Cache>,
}

impl<'a> Transaction<'a> {
    pub(crate) fn new(inner: Box<dyn StorageTransaction + 'a>) -> Self {
        Self {
            inner,
            cache: RefCell::default(),
        }
    }

    pub fn create<T: Object>(&self, obj: T) -> Result<Tx<'_, T>> {
        self.create_if_not_exists::<T>()?;

        let node = Rc::new(ObjectNode {
            id: self.inner.insert_row(T::schema(), &obj.as_table_row())?,
            state: Cell::new(ObjectState::Clean),
            obj: RefCell::new(Box::new(obj)),
        });
        self.cache.borrow_mut().insert(node.clone());

        Ok(Tx {
            lifetime: PhantomData,
            node,
        })
    }

    pub fn get<T: Object>(&self, id: ObjectId) -> Result<Tx<'_, T>> {
        if let Some(node) = self.cache.borrow().get::<T>(id) {
            if let ObjectState::Removed = node.state.get() {
                return Err(Error::NotFound(Box::new(NotFoundError {
                    object_id: id,
                    type_name: T::schema().type_name,
                })));
            }

            return Ok(Tx {
                lifetime: PhantomData,
                node,
            });
        }

        self.create_if_not_exists::<T>()?;

        let node = Rc::new(ObjectNode {
            id,
            state: Cell::new(ObjectState::Clean),
            obj: RefCell::new(Box::new(T::from_table_row(
                self.inner.select_row(id, T::schema())?,
            ))),
        });
        self.cache.borrow_mut().insert(node.clone());

        Ok(Tx {
            lifetime: PhantomData,
            node,
        })
    }

    fn create_if_not_exists<T: Object>(&self) -> Result<()> {
        if !self.inner.table_exists(T::schema().table_name)? {
            self.inner.create_table(T::schema())?;
        }

        Ok(())
    }

    pub fn commit(self) -> Result<()> {
        for node in self.cache.borrow().iter_nodes() {
            let obj = node.obj.borrow();
            match node.state.get() {
                ObjectState::Clean => (),
                ObjectState::Modified => {
                    self.inner
                        .update_row(node.id, obj.schema(), &obj.as_table_row())?;
                }
                ObjectState::Removed => {
                    self.inner.delete_row(node.id, obj.schema())?;
                }
            }
        }

        self.inner.commit()?;
        Ok(())
    }

    pub fn rollback(self) -> Result<()> {
        self.inner.rollback()?;
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy)]
pub enum ObjectState {
    Clean,
    Modified,
    Removed,
}

#[derive(Clone)]
pub struct Tx<'a, T> {
    lifetime: PhantomData<&'a T>,
    node: Rc<ObjectNode>,
}

impl<'a, T: Any> Tx<'a, T> {
    pub fn id(&self) -> ObjectId {
        self.node.id
    }

    pub fn state(&self) -> ObjectState {
        self.node.state.get()
    }

    pub fn borrow(&self) -> Ref<'_, T> {
        if let ObjectState::Removed = self.state() {
            panic!("cannot borrow a removed object");
        }

        Ref::map(self.node.obj.borrow(), |node| {
            node.as_any().downcast_ref().unwrap()
        })
    }

    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        match self.state() {
            ObjectState::Clean => self.node.state.set(ObjectState::Modified),
            ObjectState::Modified => (),
            ObjectState::Removed => panic!("cannot borrow a removed object"),
        }

        RefMut::map(self.node.obj.borrow_mut(), |node| {
            node.as_mut_any().downcast_mut().unwrap()
        })
    }

    pub fn delete(self) {
        if self.node.obj.try_borrow_mut().is_err() {
            panic!("cannot delete a borrowed object");
        }
        self.node.state.set(ObjectState::Removed);
    }
}

struct ObjectNode {
    obj: RefCell<Box<dyn Store>>,
    id: ObjectId,
    state: Cell<ObjectState>,
}

#[derive(Default)]
struct Cache(HashMap<(TypeId, ObjectId), Rc<ObjectNode>>);

impl Cache {
    pub fn insert(&mut self, node: Rc<ObjectNode>) {
        let type_id = node.obj.borrow().as_any().type_id();
        self.0.insert((type_id, node.id), node);
    }

    pub fn get<T: Any>(&self, obj_id: ObjectId) -> Option<Rc<ObjectNode>> {
        self.0.get(&(TypeId::of::<T>(), obj_id)).cloned()
    }

    pub fn iter_nodes(&self) -> impl Iterator<Item = &Rc<ObjectNode>> {
        self.0.values()
    }
}
