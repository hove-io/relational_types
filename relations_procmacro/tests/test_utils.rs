use typed_index_collection::*;

pub struct Animal {
    pub id: String,
}
impl_id!(Animal);

pub struct Feline {
    pub id: String,
    pub animal_id: String,
}
impl_id!(Feline);
impl_id!(Feline, Animal, animal_id);

pub struct Cat {
    pub id: String,
    pub feline_id: String,
}
impl_id!(Cat);
impl_id!(Cat, Feline, feline_id);
