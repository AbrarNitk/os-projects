use std::ptr::NonNull;

pub type Link<T> = Option<NonNull<Node<T>>>;

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

impl<T> Node<T> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            next: None,
            prev: None,
        }
    }
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let mut node_ptr =
            unsafe { NonNull::new_unchecked(Box::into_raw(Box::new(Node::new(elem)))) };
        if let Some(mut head) = self.head {
            // put the new node in front of the old node
            unsafe {
                node_ptr.as_mut().next = Some(head);
                head.as_mut().prev = Some(node_ptr);
                self.head = Some(node_ptr);
            };
        } else {
            // put the new node as it is
            self.head = Some(node_ptr);
            self.tail = Some(node_ptr);
        }
        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if let Some(tail) = self.tail {
            let node = unsafe { Box::from_raw(tail.as_ptr()) };

            if let Some(mut pre) = node.prev {
                self.tail = Some(pre);
                unsafe { pre.as_mut().next = None };
            } else {
                self.head = None;
                self.tail = None;
            }
            Some(node.data)
        } else {
            None
        }
    }
}
