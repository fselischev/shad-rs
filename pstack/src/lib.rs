#![forbid(unsafe_code)]

use std::rc::Rc;

// compose::begin_private(no_hint)
////////////////////////////////////////////////////////////////////////////////

struct Node<T> {
    value: Rc<T>,
    next: Option<Rc<Node<T>>>,
}

// compose::end_private
////////////////////////////////////////////////////////////////////////////////

pub struct PStack<T> {
    // compose::begin_private
    head: Option<Rc<Node<T>>>,
    len: usize,
    // compose::end_private
}

impl<T> Default for PStack<T> {
    fn default() -> Self {
        Self { head: None, len: 0 } // compose::private(unimplemented)
    }
}

impl<T> Clone for PStack<T> {
    fn clone(&self) -> Self {
        // compose::begin_private(unimplemented)
        Self {
            head: self.head.clone(),
            len: self.len,
        }
        // compose::end_private
    }
}

impl<T> PStack<T> {
    pub fn new() -> Self {
        Self::default() // compose::private(unimplemented)
    }

    pub fn push(&self, value: T) -> Self {
        // compose::begin_private(unimplemented)
        Self {
            head: Some(Rc::new(Node {
                value: Rc::new(value),
                next: self.head.clone(),
            })),
            len: self.len + 1,
        }
        // compose::end_private
    }

    pub fn pop(&self) -> Option<(Rc<T>, Self)> {
        // compose::begin_private(unimplemented)
        self.head.as_ref().map(|node| {
            (
                Rc::clone(&node.value),
                Self {
                    head: node.next.clone(),
                    len: self.len - 1,
                },
            )
        })
        // compose::end_private
    }

    pub fn len(&self) -> usize {
        self.len // compose::private(unimplemented)
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0 // compose::private(unimplemented)
    }

    pub fn iter(&self) -> impl Iterator<Item = Rc<T>> {
        // compose::begin_private(unimplemented)
        PStackIter {
            next: self.head.clone(),
        }
        // compose::end_private
    }
}

// compose::begin_private(no_hint)
pub struct PStackIter<T> {
    next: Option<Rc<Node<T>>>,
}

impl<T> Iterator for PStackIter<T> {
    type Item = Rc<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take();
        if let Some(node) = next {
            self.next = node.next.clone();
            Some(Rc::clone(&node.value))
        } else {
            None
        }
    }
}
// compose::end_private
