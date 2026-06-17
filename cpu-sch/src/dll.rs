use std::ptr::NonNull;

pub type Link<T> = NonNull<Node<T>>;

pub struct LinkedList<T> {
    pub head: Link<T>,
    pub tail: Link<T>,
    pub len: usize,
}

pub struct Node<T> {
    pub data: T,
    pub prev: Link<T>,
    pub next: Link<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: NonNull::dangling(),
            tail: NonNull::dangling(),
            len: 0,
        }
    }
}
