use std::collections::HashMap;

/// # Problem
/// Given a string `s`, find the length of the **longest** substring
/// without duplicate characters.
///
/// # Examples
/// Input: s = "abcabcbb"
/// Output: 3
/// Why? "abc" or "bca" or "cab" are correct.
///
/// Input: s = "bbbbb"
/// Output: 1
/// Why? "b" is the longest substring.
///
/// Input: s = "pwwkew"
/// Output: 3
/// Why? "wke" is the longest substring.
///
/// Input: s = " "
/// Output: 1
///
/// Input: s = "dvdf"
/// Output: 3
/// Why? "vdf" is the longest substring.
///
/// # Solution
/// We keep track of the length of the substring as a window between two
/// indices, and which chars have been seen by using a hashmap pointing
/// to the index in which it was last seen. If a char is already present
/// in the hashmap we move the left pointer to the index position at which
/// it was present.
///
/// +-----------------------+
/// | a | b | c | a | d | e |
/// +-----------------------+
///   |
///  left
///
/// The left pointer will stay there until we find the other 'a', at which
/// point we will move it but it won't change our maximum substring.
/// +-----------------------+
/// | a | b | c | a | d | e |
/// +-----------------------+
///               |
///              left
///  
/// Either the substring 'abc' or 'ade' are correct (length 3).
///
/// # Complexity analysis
///  - Time: `O(n)`
///  - Space: `O(n)`
///
/// # Performance
///  - Runtime: 0 ms | Beats 100.00%
///  - Memory: 2.35 MB | Beats 37.95%
pub fn length_of_longest_substring(s: String) -> i32 {
    let mut c2i = HashMap::new();
    let mut res: usize = 0;
    let mut lidx: usize = 0;

    for (i, c) in s.chars().enumerate() {
        if let Some(p) = c2i.get(&c)
            && *p >= lidx
        {
            lidx = p + 1;
        };

        c2i.insert(c, i);
        let sstr_len = i - lidx + 1;
        if sstr_len > res {
            res = sstr_len;
        }
    }

    res as i32
}

fn main() {
    println!("{}", length_of_longest_substring("abcabcbb".to_string()));
    println!("{}", length_of_longest_substring("bbbbb".to_string()));
    println!("{}", length_of_longest_substring("pwwkew".to_string()));
    println!("{}", length_of_longest_substring(" ".to_string()));
    println!("{}", length_of_longest_substring("dvdf".to_string()));
}
