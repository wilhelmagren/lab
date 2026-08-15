trait SearchIndex {
    fn query(&self, qs: &[u32]) -> Vec<u32>;
    fn query_one(&self, q: u32) -> u32;
}

fn main() {
    println!("Hello, world!");
}
