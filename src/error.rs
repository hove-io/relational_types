use thiserror::Error;

#[derive(Error, Debug)]
/// Typed error for `collections`.
pub enum Error {
    /// This error occurs when an identifier is not in a `CollectionWithId`.
    #[error("identifier {0} not found while building relation {1}")]
    IdentifierNotFound(String, String),
}
