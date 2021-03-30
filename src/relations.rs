use crate::Error;
use derivative::Derivative;
use std::collections::{BTreeMap, BTreeSet};
use typed_index_collection::{CollectionWithId, Id, Idx};

/// The corresponding result type used by the crate.
type Result<T, E = Error> = std::result::Result<T, E>;

/// A set of `Idx<T>`
pub type IdxSet<T> = BTreeSet<Idx<T>>;

#[macro_export]
/// Creates an IdxSet
///
/// ```no_run
/// # use relational_types::idx_set;
/// # use typed_index_collection::Idx;
/// struct Object;
/// # fn get_object_idx() -> Idx<Object> { unimplemented!() }
/// let idx: Idx<Object> = get_object_idx();
/// let idx_set = idx_set![idx];
/// ```
macro_rules! idx_set {
    ($($idx:expr),*) => {{
        let mut idx_set = relational_types::IdxSet::default();
        $(
            idx_set.insert($idx);
        )*
        idx_set
    }}
}

/// An object linking 2 types together.
pub trait Relation {
    /// The type of the source object
    type From;

    /// The type of the targer object
    type To;

    /// Returns the complete set of the source objects.
    fn get_from(&self) -> IdxSet<Self::From>;

    /// Returns the complete set of the target objects.
    fn get_to(&self) -> IdxSet<Self::To>;

    /// For a given set of the source objects, returns the
    /// corresponding targets objects.
    fn get_corresponding_forward(&self, from: &IdxSet<Self::From>) -> IdxSet<Self::To>;

    /// For a given set of the target objects, returns the
    /// corresponding source objects.
    fn get_corresponding_backward(&self, from: &IdxSet<Self::To>) -> IdxSet<Self::From>;
}

/// A one to many relation, i.e. to one `T` corresponds many `U`,
/// and a `U` has one corresponding `T`.
#[derive(Derivative, Debug)]
#[derivative(Default(bound = ""))]
pub struct OneToMany<T, U> {
    one_to_many: BTreeMap<Idx<T>, IdxSet<U>>,
    many_to_one: BTreeMap<Idx<U>, Idx<T>>,
}

impl<T, U> OneToMany<T, U>
where
    T: Id<T>,
    U: Id<U> + Id<T>,
{
    /// Construct the relation automatically from the 2 given
    /// `CollectionWithId`s.
    pub fn new(
        one: &CollectionWithId<T>,
        many: &CollectionWithId<U>,
        rel_name: &str,
    ) -> Result<Self> {
        let mut one_to_many = BTreeMap::default();
        let mut many_to_one = BTreeMap::default();
        for (many_idx, obj) in many {
            let one_id = <U as Id<T>>::id(obj);
            let one_idx = one
                .get_idx(one_id)
                .ok_or_else(|| Error::IdentifierNotFound(one_id.to_owned(), rel_name.to_owned()))?;
            many_to_one.insert(many_idx, one_idx);
            one_to_many
                // First remove existing relation for 'to' if it's not related to 'from'
                .entry(one_idx)
                .or_insert_with(IdxSet::default)
                .insert(many_idx);
        }
        Ok(OneToMany {
            one_to_many,
            many_to_one,
        })
        // Then add the new relation
    }

    /// Add a new link between a 'from' object and 'to' object.
    ///
    /// ```
    /// # use relational_types::{idx_set, IdxSet, OneToMany, Relation};
    /// # use typed_index_collection::{CollectionWithId, Id};
    /// # #[derive(Debug)]
    /// # struct Brand {
    /// #     id: String,
    /// # }
    /// # impl Id<Brand> for Brand {
    /// #     fn id(&self) -> &str { self.id.as_str() }
    /// #     fn set_id(&mut self, id: String) { unimplemented!() }
    /// # }
    /// # #[derive(Debug)]
    /// # struct Bike {
    /// #     id: String,
    /// #     brand_id: String,
    /// # }
    /// # impl Id<Bike> for Bike {
    /// #     fn id(&self) -> &str { self.id.as_str() }
    /// #     fn set_id(&mut self, id: String) { unimplemented!() }
    /// # }
    /// # impl Id<Brand> for Bike {
    /// #     fn id(&self) -> &str { self.brand_id.as_str() }
    /// #     fn set_id(&mut self, id: String) { unimplemented!() }
    /// # }
    /// // Build the relation
    /// let mut brands = CollectionWithId::default();
    /// let biky_idx = brands.push(Brand {
    ///     id: "biky".to_string(),
    /// }).unwrap();
    /// let mut bikes = CollectionWithId::default();
    /// let loulou_idx = bikes.push(Bike {
    ///     id: "loulou".to_string(),
    ///     brand_id: "biky".to_string(),
    /// }).unwrap();
    /// let mut relation = OneToMany::new(&brands, &bikes, "brands_to_bikes").unwrap();
    ///
    /// // Add a new bike to the relation
    /// let fifi_idx = bikes.push(Bike {
    ///     id: "fifi".to_string(),
    ///     brand_id: "biky".to_string(),
    /// }).unwrap();
    /// relation.add_link(biky_idx, fifi_idx);
    ///
    /// // Assert the new relation has been updated
    /// assert_eq!(
    ///     relation.get_corresponding_forward(&idx_set![biky_idx]),
    ///     idx_set![loulou_idx, fifi_idx],
    /// );
    /// assert_eq!(
    ///     relation.get_corresponding_backward(&idx_set![fifi_idx]),
    ///     idx_set![biky_idx],
    /// );
    ///
    /// // Add a new brand/bike to the relation
    /// let biclou_idx = brands.push(Brand {
    ///     id: "biclou".to_string()
    /// }).unwrap();
    /// let riri_idx = bikes.push(Bike {
    ///     id: "riri".to_string(),
    ///     brand_id: "biclou".to_string(),
    /// }).unwrap();
    /// relation.add_link(biclou_idx, riri_idx);
    ///
    /// // Assert the new relation has been updated
    /// assert_eq!(
    ///     relation.get_corresponding_forward(&idx_set![biclou_idx]),
    ///     idx_set![riri_idx],
    /// );
    /// assert_eq!(
    ///     relation.get_corresponding_backward(&idx_set![riri_idx]),
    ///     idx_set![biclou_idx],
    /// );
    ///
    /// // Change an existing relation (fifi was a biky)
    /// relation.add_link(biclou_idx, fifi_idx);
    ///
    /// // Assert the new relations
    /// assert_eq!(
    ///     relation.get_corresponding_forward(&idx_set![biclou_idx]),
    ///     idx_set![fifi_idx, riri_idx],
    /// );
    /// assert_eq!(
    ///     relation.get_corresponding_backward(&idx_set![fifi_idx]),
    ///     idx_set![biclou_idx],
    /// );
    /// // Assert other relations are also updated (fifi is not in biky anymore)
    /// assert_eq!(
    ///     relation.get_corresponding_forward(&idx_set![biky_idx]),
    ///     idx_set![loulou_idx],
    /// );
    /// assert_eq!(
    ///     relation.get_corresponding_backward(&idx_set![loulou_idx]),
    ///     idx_set![biky_idx],
    /// );
    /// ```
    ///
    /// # Important
    ///
    /// The caller need to ensure that the object referenced by `Idx<U>`
    /// has a real relation with the object referenced by `Idx<T>`.
    /// If not, then `Relation` might return Undefined Behavior.
    pub fn add_link(&mut self, from: Idx<T>, to: Idx<U>) {
        // First remove existing relation for 'to' if it's not related to 'from'
        if let Some(existing_from_idx) = self.many_to_one.get(&to).copied() {
            if existing_from_idx != from {
                self.many_to_one.remove_entry(&to);
                self.one_to_many.entry(existing_from_idx).and_modify(|set| {
                    set.remove(&to);
                });
            }
        }
        // Then add the new relation
        self.one_to_many
            .entry(from)
            .or_insert_with(IdxSet::default)
            .insert(to);
        self.many_to_one.insert(to, from);
    }
}

impl<T, U> Relation for OneToMany<T, U> {
    type From = T;
    type To = U;
    fn get_from(&self) -> IdxSet<T> {
        self.one_to_many.keys().cloned().collect()
    }
    fn get_to(&self) -> IdxSet<U> {
        self.many_to_one.keys().cloned().collect()
    }
    fn get_corresponding_forward(&self, from: &IdxSet<T>) -> IdxSet<U> {
        get_corresponding(&self.one_to_many, from)
    }
    fn get_corresponding_backward(&self, from: &IdxSet<U>) -> IdxSet<T> {
        from.iter()
            .filter_map(|from_idx| self.many_to_one.get(from_idx))
            .cloned()
            .collect()
    }
}

/// A many to many relation, i.e. a `T` can have multiple `U`, and
/// vice versa.
#[derive(Default, Debug)]
pub struct ManyToMany<T, U> {
    forward: BTreeMap<Idx<T>, IdxSet<U>>,
    backward: BTreeMap<Idx<U>, IdxSet<T>>,
}

impl<T, U> ManyToMany<T, U> {
    /// Constructor from the forward relation.
    pub fn from_forward(forward: BTreeMap<Idx<T>, IdxSet<U>>) -> Self {
        let mut backward = BTreeMap::default();
        forward
            .iter()
            .flat_map(|(&from_idx, obj)| obj.iter().map(move |&to_idx| (from_idx, to_idx)))
            .for_each(|(from_idx, to_idx)| {
                backward
                    .entry(to_idx)
                    .or_insert_with(IdxSet::default)
                    .insert(from_idx);
            });
        ManyToMany { forward, backward }
    }

    /// Constructor from 2 chained relations, i.e. from the relations
    /// `A->B` and `B->C`, constructs the relation `A->C`.
    pub fn from_relations_chain<R1, R2>(r1: &R1, r2: &R2) -> Self
    where
        R1: Relation<From = T>,
        R2: Relation<From = R1::To, To = U>,
    {
        let forward = r1
            .get_from()
            .into_iter()
            .map(|idx| {
                let from = Some(idx).into_iter().collect();
                let tmp = r1.get_corresponding_forward(&from);
                (idx, r2.get_corresponding_forward(&tmp))
            })
            .collect();
        Self::from_forward(forward)
    }

    /// Constructor from 2 relations with a common sink, i.e. from the
    /// relations `A->B` and `C->B`, constructs the relation `A->C`.
    pub fn from_relations_sink<R1, R2>(r1: &R1, r2: &R2) -> Self
    where
        R1: Relation<From = T>,
        R2: Relation<From = U, To = R1::To>,
    {
        let forward = r1
            .get_from()
            .into_iter()
            .map(|idx| {
                let from = Some(idx).into_iter().collect();
                let tmp = r1.get_corresponding_forward(&from);
                (idx, r2.get_corresponding_backward(&tmp))
            })
            .collect();
        Self::from_forward(forward)
    }

    /// Constructor from 2 relations with a common source, i.e. from
    /// the relations `B->A` and `B->C`, constructs the relation
    /// `A->C`.
    pub fn from_relations_source<R1, R2>(r1: &R1, r2: &R2) -> Self
    where
        R1: Relation<To = T>,
        R2: Relation<From = R1::From, To = U>,
    {
        let forward = r1
            .get_to()
            .into_iter()
            .map(|idx| {
                let from = Some(idx).into_iter().collect();
                let tmp = r1.get_corresponding_backward(&from);
                (idx, r2.get_corresponding_forward(&tmp))
            })
            .collect();
        Self::from_forward(forward)
    }
}

impl<T, U> Relation for ManyToMany<T, U> {
    type From = T;
    type To = U;
    fn get_from(&self) -> IdxSet<T> {
        self.forward.keys().cloned().collect()
    }
    fn get_to(&self) -> IdxSet<U> {
        self.backward.keys().cloned().collect()
    }
    fn get_corresponding_forward(&self, from: &IdxSet<T>) -> IdxSet<U> {
        get_corresponding(&self.forward, from)
    }
    fn get_corresponding_backward(&self, from: &IdxSet<U>) -> IdxSet<T> {
        get_corresponding(&self.backward, from)
    }
}

fn get_corresponding<T, U>(map: &BTreeMap<Idx<T>, IdxSet<U>>, from: &IdxSet<T>) -> IdxSet<U> {
    from.iter()
        .filter_map(|from_idx| map.get(from_idx))
        .flat_map(|indices| indices.iter().cloned())
        .collect()
}
