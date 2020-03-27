mod test_utils;

use relational_types::*;
use test_utils::*;
use typed_index_collection::*;

#[derive(GetCorresponding)]
pub struct Model {
    #[get_corresponding(nonsupportedargument)]
    animal_to_felines: OneToMany<Animal, Feline>,
}

fn main() {}
