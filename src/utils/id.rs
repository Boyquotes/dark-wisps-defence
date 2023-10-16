use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops::Deref;

#[derive(Debug, Eq, PartialEq)]
pub struct Id<T, IdType> {
    id: IdType,
    phantom: PhantomData<T>,
}

impl<T, IdType> Id<T, IdType> {
    pub fn new(id: IdType) -> Self {
        Id { id, phantom: PhantomData }
    }
}

impl<T, IdType> From<IdType> for Id<T, IdType> {
    fn from(value: IdType) -> Self {
        Id { id: value, phantom: PhantomData }
    }
}

impl <T, IdType> Clone for Id<T, IdType> where IdType: Clone {
    fn clone(&self) -> Id<T, IdType> {
        Id { id: self.id.clone(), phantom: PhantomData }
    }
}
impl <T, IdType>Copy for Id<T, IdType> where IdType: Copy {}

impl <T, IdType>Deref for Id<T, IdType> {
    type Target = IdType;

    fn deref(&self) -> &Self::Target {
        &self.id
    }
}

impl <T, IdType>Hash for Id<T, IdType> where IdType: Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl <T, IdType>Display for Id<T, IdType> where IdType: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}