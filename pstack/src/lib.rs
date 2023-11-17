#![forbid(unsafe_code)]

use std::rc::Rc;

// Define a reference-counted node for the stack
struct Node<T> {
    value: T,
    next: Option<Rc<Node<T>>>,
}

pub struct PRef<T> {
    node: Rc<Node<T>>,
}

impl<T> std::ops::Deref for PRef<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.node.value
    }
}

pub struct PStack<T> {
    head: Option<Rc<Node<T>>>,
    size: usize,
}

impl<T> Default for PStack<T> {
    fn default() -> Self {
        PStack::new()
    }
}

impl<T> Clone for PStack<T> {
    fn clone(&self) -> Self {
        PStack {
            head: self.head.clone(),
            size: self.size,
        }
    }
}

impl<T> PStack<T> {
    pub fn new() -> Self {
        PStack {
            head: None,
            size: 0,
        }
    }

    pub fn push(&self, value: T) -> Self {
        let new_node = Rc::new(Node {
            value,
            next: self.head.clone(),
        });

        PStack {
            head: Some(new_node),
            size: self.size + 1,
        }
    }

    pub fn pop(&self) -> Option<(PRef<T>, Self)> {
        match self.head {
            Some(ref head) => {
                let new_head = head.next.clone();
                let popped = PRef { node: head.clone() };
                Some((
                    popped,
                    PStack {
                        head: new_head,
                        size: self.size - 1,
                    },
                ))
            }
            None => None,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = PRef<T>> {
        Iter {
            current: self.head.clone(),
        }
    }
}

struct Iter<T> {
    current: Option<Rc<Node<T>>>,
}

impl<T> Iterator for Iter<T> {
    type Item = PRef<T>;

    fn next(&mut self) -> Option<PRef<T>> {
        if let Some(node) = self.current.clone() {
            self.current = node.next.clone();
            return Some(PRef { node: node.clone() });
        }
        None
    }
}
