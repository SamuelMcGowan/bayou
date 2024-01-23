use super::NodeCopy;

#[derive(NodeCopy!)]
pub struct Place(pub usize);

#[derive(NodeCopy!)]
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

#[derive(NodeCopy!)]
pub enum Ownership {
    Owned,
    Borrowed,
}
