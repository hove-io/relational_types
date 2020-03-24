mod test_utils;

use relations::*;
use relations_procmacro::*;
use test_utils::*;
use typed_index_collection::*;

#[derive(GetCorresponding)]
pub struct Model {
    animals_to_felines: OneToMany<Animal, Feline>,
    felines_to_cats: OneToMany<Feline, Cat>,
}

fn main() {
    let animal = Animal {
        id: String::from("animal_id"),
    };
    let feline = Feline {
        id: String::from("feline_id"),
        animal_id: String::from("animal_id"),
    };
    let animals = CollectionWithId::from(animal);
    let felines = CollectionWithId::from(feline);
    let cats = CollectionWithId::<Cat>::default();
    let model = Model {
        animals_to_felines: OneToMany::new(&animals, &felines, "animals_to_felines").unwrap(),
        felines_to_cats: OneToMany::new(&felines, &cats, "felines_to_cats").unwrap(),
    };

    let animal_idx = animals.get_idx("animal_id").unwrap();
    let cat_indexes: IdxSet<Cat> = model.get_corresponding_from_idx(animal_idx);
    assert_eq!(0, cat_indexes.len());
}
