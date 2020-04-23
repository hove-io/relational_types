mod test_utils;

use relational_types::*;
use test_utils::*;

#[derive(GetCorresponding)]
pub struct Model {
    #[get_corresponding(weight = "abc")]
    animals_to_felines: OneToMany<Animal, Feline>,
}

fn main() {}
