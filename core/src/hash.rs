use std::hash::Hash;
use crate::ops::*;
use tract_linalg::hash::DynHash;

impl Hash for Box<dyn Op> {
    fn hash<H: std::hash::Hasher>(&self, mut state: &mut H) {
        std::hash::Hash::hash(&self.type_id(), state);
        DynHash::dyn_hash(self, &mut state)
    }
}

impl<'a> Hash for &'a dyn Op {
    fn hash<H: std::hash::Hasher>(&self, mut state: &mut H) {
        std::hash::Hash::hash(&self.type_id(), state);
        DynHash::dyn_hash(self, &mut state)
    }
}

impl Hash for Box<dyn TypedOp> {
    fn hash<H: std::hash::Hasher>(&self, mut state: &mut H) {
        std::hash::Hash::hash(&self.type_id(), state);
        DynHash::dyn_hash(self, &mut state)
    }
}

impl Hash for Box<dyn PulsedOp> {
    fn hash<H: std::hash::Hasher>(&self, mut state: &mut H) {
        std::hash::Hash::hash(&self.type_id(), state);
        DynHash::dyn_hash(self, &mut state)
    }
}

impl<'a> Hash for &'a dyn PulsedOp {
    fn hash<H: std::hash::Hasher>(&self, mut state: &mut H) {
        std::hash::Hash::hash(&self.type_id(), state);
        DynHash::dyn_hash(self, &mut state)
    }
}
