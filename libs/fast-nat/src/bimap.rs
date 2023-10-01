use std::hash::Hash;

use rustc_hash::FxHashMap;

/// A bi-directional hash map
#[derive(Debug, Clone)]
pub struct BiHashMap<Left, Right> {
    /// Mapping from a left value to a right value
    left_to_right: FxHashMap<Left, Right>,
    /// Mapping from a right value to a left value
    right_to_left: FxHashMap<Right, Left>,
}

impl<Left, Right> BiHashMap<Left, Right>
where
    Left: Eq + Hash + Clone,
    Right: Eq + Hash + Clone,
{
    /// Construct a new empty `BiHashMap`
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a new mapping into the `BiHashMap`
    #[profiling::function]
    pub fn insert(&mut self, left: Left, right: Right) {
        self.left_to_right.insert(left.clone(), right.clone());
        self.right_to_left.insert(right, left);
    }

    /// Get the right value for a given left value
    #[profiling::function]
    pub fn get_right(&self, left: &Left) -> Option<&Right> {
        self.left_to_right.get(left)
    }

    /// Get the left value for a given right value
    #[profiling::function]
    pub fn get_left(&self, right: &Right) -> Option<&Left> {
        self.right_to_left.get(right)
    }

    /// Remove a mapping from the `BiHashMap`
    #[profiling::function]
    pub fn remove(&mut self, left: &Left, right: &Right) {
        self.left_to_right.remove(left);
        self.right_to_left.remove(right);
    }

    /// Remove a mapping from the `BiHashMap` by left value
    #[profiling::function]
    pub fn remove_left(&mut self, left: &Left) {
        if let Some(right) = self.left_to_right.remove(left) {
            self.right_to_left.remove(&right);
        }
    }

    /// Remove a mapping from the `BiHashMap` by right value
    #[profiling::function]
    pub fn remove_right(&mut self, right: &Right) {
        if let Some(left) = self.right_to_left.remove(right) {
            self.left_to_right.remove(&left);
        }
    }

    /// Get the total number of mappings in the `BiHashMap`
    #[profiling::function]
    pub fn len(&self) -> usize {
        self.left_to_right.len()
    }

    /// Check if the `BiHashMap` is empty
    #[profiling::function]
    pub fn is_empty(&self) -> bool {
        self.left_to_right.is_empty()
    }
}

impl<Left, Right> Default for BiHashMap<Left, Right> {
    fn default() -> Self {
        Self {
            left_to_right: FxHashMap::default(),
            right_to_left: FxHashMap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut bimap = BiHashMap::new();
        bimap.insert(1, "one");
        bimap.insert(2, "two");
        bimap.insert(3, "three");

        assert_eq!(bimap.get_right(&1), Some(&"one"));
        assert_eq!(bimap.get_right(&2), Some(&"two"));
        assert_eq!(bimap.get_right(&3), Some(&"three"));

        assert_eq!(bimap.get_left(&"one"), Some(&1));
        assert_eq!(bimap.get_left(&"two"), Some(&2));
        assert_eq!(bimap.get_left(&"three"), Some(&3));
    }

    #[test]
    fn test_remove() {
        let mut bimap = BiHashMap::new();
        bimap.insert(1, "one");
        bimap.insert(2, "two");
        bimap.insert(3, "three");

        bimap.remove(&1, &"one");
        assert_eq!(bimap.get_right(&1), None);
        assert_eq!(bimap.get_left(&"one"), None);

        bimap.remove_left(&2);
        assert_eq!(bimap.get_right(&2), None);
        assert_eq!(bimap.get_left(&"two"), None);

        bimap.remove_right(&"three");
        assert_eq!(bimap.get_right(&3), None);
        assert_eq!(bimap.get_left(&"three"), None);
    }
}
