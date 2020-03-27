//! Modeling the relations between objects.
//!
//! By default, feature `relational_types_procmacro` is enabled, exposing macros to
//! help build relations. See documentation of the crate `relational_types_procmacro`
//! for more information.
//!
//! This module defines types for modeling the relations between
//! objects, and use them thanks to the `GetCorresponding` custom
//! derive.
//!
//! Let's clarify that with an example. Suppose that `Bike`s have a
//! `Brand`. `Bike`s also have an `Owner`, and these `Owner`s have a
//! `Job`. `Bike`s also have a `Kind`.
//!
//! ```raw
//! Brand - Bike - Owner - Job
//!          |
//!         Kind
//! ```
//!
//! Let's defines these relations and use them a bit:
//!
//! ```no_run
//! # use relational_types_procmacro::*;
//! # use relational_types::*;
//! # use typed_index_collection::Idx;
//! # struct Bike;
//! # struct Brand;
//! # struct Owner;
//! # struct Job;
//! # struct Kind;
//! # fn get_mbk_brand() -> Idx<Brand> { unimplemented!() }
//! #[derive(Default, GetCorresponding)]
//! pub struct World {
//!     brands_to_bikes: OneToMany<Brand, Bike>,
//!     owners_to_bikes: OneToMany<Owner, Bike>,
//!     jobs_to_owners: OneToMany<Job, Owner>,
//!     kinds_to_bikes: OneToMany<Kind, Bike>,
//! }
//! let world = World::default();
//! let mbk: Idx<Brand> = get_mbk_brand();
//! let owners_with_mbk: IdxSet<Owner> = world.get_corresponding_from_idx(mbk);
//! let jobs_with_mbk: IdxSet<Job> = world.get_corresponding(&owners_with_mbk);
//! println!(
//!    "{} owners with {} different jobs own a bike of the brand MBK.",
//!    owners_with_mbk.len(),
//!    jobs_with_mbk.len()
//! );
//! ```
//!
//! First, we want to model the relations between the object. One bike
//! has a brand, and a brand has several bikes (hopefully). Thus, we
//! use a `OneToMany<Bike, Brand>` to model this relation.
//!
//! We repeat this process to model every relation. We obtain without
//! too much effort the `World` struct.
//!
//! The `GetCorresponding` derive looks at each field of the `World`
//! struct, keeping the fields containing `_to_` with a type with 2
//! generics, and interpret that as a relation. For example,
//! `bikes_to_brands: OneToMany<Bike, Brand>` is a relation between
//! `Bike` and `Brand`. Using all the relations, it generates a graph,
//! compute the shortest path between all the types, and generate an
//! `impl GetCorresponding` for each feasible path.
//!
//! These `impl GetCorresponding` are used by
//! `World::get_corresponding_from_idx` and `World::get_corresponding`
//! that are helpers to explore the `World`.
//!
//! Thus, when we call `world.get_corresponding_from_idx(mbk)` for
//! `Owner`, we will use the generated code that, basically, gets all
//! the `Bike`s corresponding to the `Brand` MBK, and then gets all
//! the `Owner`s corresponding to these `Bike`s.
//!
//! Imagine that, in our application, we use a lot the `Owner->Kind`
//! and `Brand->Kind` search.  To do these searches, we pass by
//! `Bike`, and there is a lot of `Bike`s in our model.  Thus, as an
//! optimization, we want to precompute these relations.
//!
//! ```raw
//! Brand - Bike - Owner - Job
//!    \     |      /
//!     `-- Kind --'
//! ```
//!
//! The shortcuts `Brand - Kind` and `Kind - Owner` allow our
//! optimization, but we now have a problem for the `Owner->Brand`
//! search: we can do `Owner->Kind->Brand` and `Owner->Bike->Brand`
//! with a cost of 2.  The first solution is clearly wrong, introduced
//! by our shortcuts.  To fix this problem, we can put a weight of 1.9
//! on `Brand - Kind` and `Kind - Owner`.  The path
//! `Owner->Kind->Brand` now cost 3.8 and is discarded.
//!
//! Let's implement that:
//!
//! ```
//! # use relational_types_procmacro::*;
//! # use relational_types::*;
//! # use typed_index_collection::Idx;
//! # struct Bike;
//! # struct Brand;
//! # struct Owner;
//! # struct Job;
//! # struct Kind;
//! # fn get_mbk_brand() -> Idx<Brand> { unimplemented!() }
//! #[derive(GetCorresponding)]
//! pub struct World {
//!     brands_to_bikes: OneToMany<Brand, Bike>,
//!     owners_to_bikes: OneToMany<Owner, Bike>,
//!     jobs_to_owners: OneToMany<Job, Owner>,
//!     kinds_to_bikes: OneToMany<Kind, Bike>,
//!
//!     // shortcuts
//!     #[get_corresponding(weight = "1.9")]
//!     brands_to_kinds: ManyToMany<Brand, Kind>,
//!     #[get_corresponding(weight = "1.9")]
//!     kinds_to_owners: ManyToMany<Kind, Owner>,
//! }
//! # fn create_brands_to_bikes() -> OneToMany<Brand, Bike> { unimplemented!() }
//! # fn create_owners_to_bikes() -> OneToMany<Owner, Bike> { unimplemented!() }
//! # fn create_jobs_to_owners() -> OneToMany<Job, Owner> { unimplemented!() }
//! # fn create_kinds_to_bikes() -> OneToMany<Kind, Bike> { unimplemented!() }
//! impl World {
//!     fn new() -> World {
//!         let brands_to_bikes = create_brands_to_bikes();
//!         let owners_to_bikes = create_owners_to_bikes();
//!         let jobs_to_owners = create_jobs_to_owners();
//!         let kinds_to_bikes = create_kinds_to_bikes();
//!         World {
//!             brands_to_kinds: ManyToMany::from_relations_sink(
//!                 &brands_to_bikes,
//!                 &kinds_to_bikes,
//!             ),
//!             kinds_to_owners: ManyToMany::from_relations_sink(
//!                 &kinds_to_bikes,
//!                 &owners_to_bikes,
//!             ),
//!             brands_to_bikes,
//!             owners_to_bikes,
//!             jobs_to_owners,
//!             kinds_to_bikes,
//!         }
//!     }
//! }
//! ```

mod error;
mod relations;

pub use crate::error::*;
pub use crate::relations::*;
#[cfg(feature = "relational_types_procmacro")]
pub use relational_types_procmacro::*;
