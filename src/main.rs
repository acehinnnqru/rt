use std::str::FromStr;

mod repository;
use repository::Repository;

mod config;

fn main() {
    let path = std::env::args().nth(1).expect("no repository given");

    let repo = Repository::from_str(&path).unwrap();

    println!("\nparsed repository: {:?}\n", repo);

    config::init();

    let root = config::root();

    if root.trim().is_empty() {
        unreachable!("invalide root setting")
    }

    println!("root: {}", root);
}
