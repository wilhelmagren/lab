#[derive(Clone, Debug, Eq, PartialEq)]
struct ListNode {
    pub val: i32,
    pub next: Option<Box<ListNode>>,
}

impl ListNode {
    fn new(val: i32) -> Self {
        Self { val, next: None }
    }
}

type LinkedList = Option<Box<ListNode>>;

/// # Problem
/// You are given two **non-empty** linked lists representing two non-negative
/// integers. The digits are stored in **reverse order**, and each of their nodes
/// contains a single digit. Add the two numbers and return the sum as a linked list.
///
/// You may assume the two numbers do not contain any leading zero, except the number 0 itself.
///
/// # Examples
/// Input: l1 = [2,4,3], l2 = [5,6,4]
/// Output: [7,0,8]
/// Why? 342 + 465 = 807
///
/// Input: l1 = [0], l2 = [0]
/// Output: [0]
///
/// Input: l1 = [9,9,9,9,9,9,9], l2 = [9,9,9,9]
/// Output: [8,9,9,9,0,0,0,1]
///
/// # Solution
/// Go through the ListNode's in pairs and add each number. Keep track
/// of a carry for the next node pairs sum. If one linked list is
/// longer than the other we just pretend there is a node there with
/// value 0. Always take Euclidean remainder (base 10) of the sum.
/// We start with a dummy sentinel node as the base for the linked list.
///
/// # Complexity analysis
/// Given that `k = max(n, m)` where `n` and `m` are the lengths of the linked lists:
///  - Time: `O(k)` assume heap allocation is `O(1)`.
///  - Space: `O(k)` because we build a new resulting linked list.
///
/// # Performance
///  - Runtime: 0 ms | Beats 100.00%
///  - Memory: 2.17 MiB | Beats 94.99%
fn add_two_numbers(l1: LinkedList, l2: LinkedList) -> LinkedList {
    let mut l1 = l1;
    let mut l2 = l2;

    let mut dummy = ListNode::new(0);
    let mut curr = &mut dummy;

    let mut carry = 0;

    while l1.is_some() || l2.is_some() || carry != 0 {
        let v1 = if let Some(node) = l1 {
            l1 = node.next;
            node.val
        } else {
            0
        };

        let v2 = if let Some(node) = l2 {
            l2 = node.next;
            node.val
        } else {
            0
        };

        let sum = v1 + v2 + carry;
        carry = sum / 10;
        curr.next = Some(Box::new(ListNode::new(sum.rem_euclid(10))));
        curr = curr.next.as_mut().unwrap();
    }

    dummy.next
}

fn main() {
    let l13 = ListNode::new(3);
    let mut l12 = ListNode::new(4);
    let mut l11 = ListNode::new(2);
    l12.next = Some(Box::new(l13));
    l11.next = Some(Box::new(l12));

    let l23 = ListNode::new(4);
    let mut l22 = ListNode::new(6);
    let mut l21 = ListNode::new(5);
    l22.next = Some(Box::new(l23));
    l21.next = Some(Box::new(l22));

    let mut out = add_two_numbers(Some(Box::new(l11)), Some(Box::new(l21)));

    while let Some(node) = out {
        print!("{}", node.val);
        out = node.next;
    }

    println!();
}
