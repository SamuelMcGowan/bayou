#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Place(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlaceRef {
    pub place: Place,
    pub ownership: Ownership,
}

impl PlaceRef {
    pub fn owned(place: Place) -> Self {
        Self {
            place,
            ownership: Ownership::Owned,
        }
    }

    pub fn borrowed(place: Place) -> Self {
        Self {
            place,
            ownership: Ownership::Borrowed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ownership {
    Owned,
    Borrowed,
}
