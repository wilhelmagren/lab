use std::default::Default;
use std::fmt;

pub type NodeRef<T> = Box<Node<T>>;

/// What is a linked list? It's a bunch of data on the heap that point
/// to each other in sequence.
///
/// Ask a functional programmer and the will probably tell you:
/// List a = Empty | Elem a (List a)
/// Which reads as "A list is either empty or an element followed by a list".
/// This is a recursive definition expressed as a sum type, which means
/// "a type that can have different values which may be different types".
///
/// In Rust, an Enum is a sum type.
#[derive(Debug, Default)]
pub struct Node<T> {
    value: T,
    next: Option<NodeRef<T>>,
}

impl<T: fmt::Debug> Node<T> {
    pub fn new(value: T) -> Self {
        Node { value, next: None }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn next(&self) -> &Option<NodeRef<T>> {
        &self.next
    }

    pub fn link(&mut self, next: NodeRef<T>) {
        self.next = Some(next);
    }

    pub fn walk_print(self) {
        let mut node = Box::new(self);
        println!("{:?}", node.value);
        while let Some(next) = node.next {
            println!("{:?}", next.value);
            node = next;
        }
    }
}

impl<T: fmt::Debug> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node<value: {:?}, next: {:?}>", self.value, self.next)
    }
}

fn main() {
    let mut n1 = Node::new("a");
    let n2 = Node::new("b");
    n1.link(Box::new(n2));

    println!("{}", n1);

    n1.walk_print();
}
