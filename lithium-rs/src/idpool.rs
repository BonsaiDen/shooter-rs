// External Dependencies ------------------------------------------------------
use std;
use std::collections::HashMap;
use num::{Zero, One, Bounded};


// Pooled, re-usable ID generator----------------------------------------------
pub struct IdPool<T> {
    acquired_ids: HashMap<T, bool>,
    released_ids: Vec<T>,
    used_ids: T,
    next_id: T
}

impl<T> IdPool<T>
    where T: Copy
          + std::cmp::PartialOrd
          + std::cmp::Eq
          + std::hash::Hash
          + std::ops::Add<T, Output = T>
          + std::ops::Sub<T, Output = T>
          + One
          + Zero
          + Bounded
{

    pub fn new() -> IdPool<T> {
        IdPool::<T> {
            acquired_ids: HashMap::new(),
            used_ids: T::zero(),
            next_id: T::zero(),
            released_ids: Vec::new()
        }
    }

    pub fn reset(&mut self) {
        self.acquired_ids.clear();
        self.used_ids = T::zero();
        self.next_id = T::zero();
        self.released_ids.clear();
    }

    pub fn get_id(&mut self) -> Option<T> {

        // Don't exceed the pool size
        if self.used_ids < T::max_value() {

            // Use released ids first this is O(1)
            let id = if !self.released_ids.is_empty() {
                self.released_ids.pop().unwrap()

            // If there are no released ids available, probe the id space
            // - O(1) best case
            // - O(n) worst case
            } else {
                while self.acquired_ids.contains_key(&self.next_id) {
                    self.next_id = self.next_id + T::one();
                    if self.next_id == T::max_value() {
                        self.next_id = T::zero();
                    }
                }
                self.next_id
            };

            // Mark the id as used
            self.acquired_ids.insert(id, true);
            self.used_ids = self.used_ids + T::one();
            Some(id)

        } else {
            None
        }

    }

    pub fn release_id(&mut self, id: T) {
        // Remove the id and push it into the released ids pool for quick
        // re-use
        if let Some(_) = self.acquired_ids.remove(&id) {
            self.used_ids = self.used_ids - T::one();
            // TODO for now we do not want to instantly re-use the IDs in
            // order to prevent re-usage network / issues
            //self.released_ids.push(id);
        }
    }

}

