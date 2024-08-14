use std::str::FromStr;

mod repository;
use repository::Repository;

fn main() {
    let repo = std::env::args().nth(1).expect("no repository given");

    println!("repository: {:?}", Repository::from_str(&repo));
}
