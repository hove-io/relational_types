mod test_utils;

use relational_types::*;
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
    let feline1 = Feline {
        id: String::from("feline_id_1"),
        animal_id: String::from("animal_id"),
    };
    let feline2 = Feline {
        id: String::from("feline_id_2"),
        animal_id: String::from("animal_id"),
    };
    let cat_1 = Cat {
        id: String::from("cat_id_1"),
        feline_id: String::from("feline_id_1"),
    };
    let cat_2 = Cat {
        id: String::from("cat_id_2"),
        feline_id: String::from("feline_id_1"),
    };
    let cat_3 = Cat {
        id: String::from("cat_id_3"),
        feline_id: String::from("feline_id_2"),
    };
    let cat_4 = Cat {
        id: String::from("cat_id_4"),
        feline_id: String::from("feline_id_2"),
    };
    let animals = CollectionWithId::from(animal);
    let felines = CollectionWithId::new(vec![feline1, feline2]).unwrap();
    let cats = CollectionWithId::new(vec![cat_1, cat_2, cat_3, cat_4]).unwrap();
    let model = Model {
        animals_to_felines: OneToMany::new(&animals, &felines, "animals_to_felines").unwrap(),
        felines_to_cats: OneToMany::new(&felines, &cats, "felines_to_cats").unwrap(),
    };

    let animal_idx = animals.get_idx("animal_id").unwrap();
    let feline_indexes: IdxSet<Feline> = model.get_corresponding_from_idx(animal_idx);
    assert_eq!(2, feline_indexes.len());
    let cat_indexes: IdxSet<Cat> = model.get_corresponding(&feline_indexes);
    let cat_1_idx = cats.get_idx("cat_id_1").unwrap();
    assert!(cat_indexes.contains(&cat_1_idx));
    let cat_2_idx = cats.get_idx("cat_id_2").unwrap();
    assert!(cat_indexes.contains(&cat_2_idx));
    let cat_3_idx = cats.get_idx("cat_id_3").unwrap();
    assert!(cat_indexes.contains(&cat_3_idx));
    let cat_4_idx = cats.get_idx("cat_id_4").unwrap();
    assert!(cat_indexes.contains(&cat_4_idx));

    let animal_indexes = model.get_corresponding_from_idx(cat_1_idx);
    assert!(animal_indexes.contains(&animal_idx));
}
