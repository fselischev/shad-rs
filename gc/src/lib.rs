#![forbid(unsafe_code)]

pub use gc_derive::Scan;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    marker::PhantomData,
    ops::Deref,
    rc::{Rc, Weak},
};

////////////////////////////////////////////////////////////////////////////////

pub struct Gc<T> {
    weak: Weak<T>,
}

impl<T> Clone for Gc<T> {
    fn clone(&self) -> Self {
        Self {
            weak: self.weak.clone(),
        }
    }
}

impl<T> Gc<T> {
    pub fn extract_addr(&self) -> usize {
        self.weak.as_ptr() as usize
    }

    pub fn borrow(&self) -> GcRef<'_, T> {
        GcRef {
            rc: self.weak.upgrade().unwrap(),
            lifetime: PhantomData::<&'_ Gc<T>>,
        }
    }
}

pub struct GcRef<'a, T> {
    rc: Rc<T>,
    lifetime: PhantomData<&'a Gc<T>>,
}

impl<'a, T> Deref for GcRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.rc
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait Scan {
    fn get_objects(&self) -> Vec<usize>;
}

impl<T: Scan + 'static> Scan for Gc<T> {
    fn get_objects(&self) -> Vec<usize> {
        vec![self.extract_addr()]
    }
}

impl<T: Scan> Scan for Vec<T> {
    fn get_objects(&self) -> Vec<usize> {
        self.iter().flat_map(|e| e.get_objects()).collect()
    }
}

impl<T: Scan> Scan for Option<T> {
    fn get_objects(&self) -> Vec<usize> {
        match self.as_ref() {
            Some(e) => e.get_objects(),
            None => vec![],
        }
    }
}

impl<T: Scan> Scan for RefCell<T> {
    fn get_objects(&self) -> Vec<usize> {
        self.borrow().get_objects()
    }
}

impl Scan for i32 {
    fn get_objects(&self) -> Vec<usize> {
        vec![]
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct Arena(Vec<Rc<dyn Scan>>);

impl Arena {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn allocation_count(&self) -> usize {
        self.0.len()
    }

    pub fn alloc<T: Scan + 'static>(&mut self, obj: T) -> Gc<T> {
        let rc: Rc<T> = Rc::new(obj);
        let weak = Rc::downgrade(&rc);
        self.0.push(rc);
        Gc { weak }
    }

    pub fn sweep(&mut self) {
        let idx_by_obj = (0..self.0.len())
            .map(|i| (Rc::as_ptr(&self.0[i]) as *const u8 as usize, i))
            .collect::<HashMap<_, _>>();
        let mut point_to = vec![0; self.0.len()];

        let graph = self
            .0
            .iter()
            .map(|a| {
                a.get_objects()
                    .iter()
                    .map(|x| {
                        point_to[idx_by_obj[x]] += 1;
                        idx_by_obj[x]
                    })
                    .collect()
            })
            .collect();

        let mut marked = HashSet::with_capacity(self.0.len());
        for (i, count) in point_to.iter().enumerate() {
            if Rc::weak_count(&self.0[i]) > *count {
                Self::mark_all(i, &mut marked, &graph);
            }
        }

        let mut j = 0;
        for i in 0..self.0.len() {
            if marked.contains(&i) {
                if i > j {
                    self.0.swap(j, i);
                }
                j += 1;
            }
        }
        self.0.truncate(j);
    }

    fn mark_all(root_addr: usize, marked: &mut HashSet<usize>, graph: &Vec<Vec<usize>>) {
        if !marked.insert(root_addr) {
            return;
        }
        for u in &graph[root_addr] {
            Self::mark_all(*u, marked, graph);
        }
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}
