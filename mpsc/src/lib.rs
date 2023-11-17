#![forbid(unsafe_code)]

use std::{cell::RefCell, collections::VecDeque, fmt::Debug, rc::Rc};
use thiserror::Error;

////////////////////////////////////////////////////////////////////////////////
///
#[derive(Default)]
pub struct Inner<T> {
    buffer: VecDeque<T>,
    state: InnerState,
}

impl<T> Inner<T> {
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            state: InnerState::default(),
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.buffer.pop_front()
    }

    pub fn push_back(&mut self, value: T) {
        self.buffer.push_back(value);
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn change_state(&mut self, state: InnerState) {
        self.state = state;
    }
}

#[derive(Default)]
pub enum InnerState {
    #[default]
    Open,
    Closed,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error("channel is closed")]
pub struct SendError<T> {
    pub value: T,
}

pub struct Sender<T> {
    inner: Rc<RefCell<Inner<T>>>,
}

impl<T> Sender<T> {
    pub fn send(&self, value: T) -> Result<(), SendError<T>> {
        if self.is_closed() {
            return Err(SendError { value });
        }
        self.inner.borrow_mut().push_back(value);
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        match self.inner.borrow().state {
            InnerState::Open => false,
            InnerState::Closed => true,
        }
    }

    pub fn same_channel(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        // first is the last tx that will be dropped and second is rx
        if Rc::strong_count(&self.inner) == 2 {
            self.inner.borrow_mut().state = InnerState::Closed;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum ReceiveError {
    #[error("channel is empty")]
    Empty,
    #[error("channel is closed")]
    Closed,
}

pub struct Receiver<T> {
    inner: Rc<RefCell<Inner<T>>>,
}

impl<T> Receiver<T> {
    pub fn recv(&mut self) -> Result<T, ReceiveError> {
        let mut buffer = self.inner.borrow_mut();
        match buffer.state {
            InnerState::Open if buffer.is_empty() => Err(ReceiveError::Empty),
            _ => buffer.pop_front().ok_or(ReceiveError::Closed),
        }
    }

    pub fn close(&mut self) {
        self.inner.borrow_mut().change_state(InnerState::Closed);
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        self.inner.borrow_mut().change_state(InnerState::Closed);
    }
}

////////////////////////////////////////////////////////////////////////////////

pub fn channel<T>() -> (Sender<T>, Receiver<T>) {
    let inner = Rc::new(RefCell::new(Inner::new()));
    (
        Sender {
            inner: Rc::clone(&inner),
        },
        Receiver { inner },
    )
}
