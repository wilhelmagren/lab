use std::collections::HashMap;

/// # Problem
/// Given an array of integers `nums` and an integer `target`,
/// return *indices of the two numbers such that they add upp to `target`*.
///
/// You may assume that each input would have **exactly one solution**,
/// and you may not use the same element twice.
///
/// You can return the answer in any order.
///
/// # Solution
/// Add each iterated number to a hashmap (number, index) and for each
/// number calculate the diff `target - curr`, if the diff is
/// in the map then we know that those two numbers added together
/// is equal to the target. We can return out early because there
/// will only be one solution.
///
/// # Comlexity analysis
///  - Time: `O(n)` assuming hashmap operations `O(1)`.
///  - Space: `O(n)` for the extra hashmap.
///
/// # Performance
///  - Runtime: 0 ms | Beats 100.0%
///  - Memory: 2.54 MB | Beats 22.43%
fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
    let mut map = HashMap::new();
    map.insert(nums[0], 0);

    for (i, n) in nums[1..].iter().enumerate() {
        let diff = target - n;
        if let Some(j) = map.get(&diff) {
            return vec![(i + 1) as i32, *j];
        }
        map.insert(*n, (i + 1) as i32);
    }

    Vec::new()
}

fn main() {
    println!("{:?}", two_sum(vec![2, 7, 11, 15], 9));
    println!("{:?}", two_sum(vec![3, 2, 4], 6));
    println!("{:?}", two_sum(vec![3, 3], 6));
}
