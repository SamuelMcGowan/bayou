#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Place(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaceRef {
    pub place: Place,
    pub ownership: Ownership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ownership {
    Owned,
    Borrowed,
}
