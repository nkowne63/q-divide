use std::collections::{HashMap, HashSet};

#[derive(Debug, Default, Clone)]
pub struct SetPartition {
    // Stores the actual subsets of elements.
    subsets: Vec<HashSet<usize>>,
    // Maps each element to the index of the subset it belongs to in the `subsets` vector.
    element_to_subset_index: HashMap<usize, usize>,
    // Used to generate unique IDs for new elements added via `add_new_element`.
    next_element_id: usize,
}

impl SetPartition {
    // Creates a new, empty SetPartition.
    pub fn new() -> Self {
        SetPartition {
            subsets: Vec::new(),
            element_to_subset_index: HashMap::new(),
            next_element_id: 0,
        }
    }

    // Adds an element to the set.
    // If the element already exists, this operation has no effect.
    // Otherwise, the element is added to a new subset containing only itself.
    pub fn add_element(&mut self, element: usize) {
        if self.element_to_subset_index.contains_key(&element) {
            // Element already exists, do nothing.
            return;
        }

        let mut new_set = HashSet::new();
        new_set.insert(element);
        self.subsets.push(new_set);
        let new_subset_index = self.subsets.len() - 1;
        self.element_to_subset_index.insert(element, new_subset_index);

        if element >= self.next_element_id {
            self.next_element_id = element + 1;
        }
    }

    // Adds a new, unique element to the set and returns its ID.
    // The new element is placed in a new subset containing only itself.
    pub fn add_new_element(&mut self) -> usize {
        let new_element = self.next_element_id;
        // Ensure the generated ID is actually new, in case `add_element` was used with higher IDs.
        // This loop is a safeguard; typically, `next_element_id` should be sufficient.
        let mut current_new_element = new_element;
        while self.element_to_subset_index.contains_key(&current_new_element) {
            current_new_element += 1;
        }
        self.next_element_id = current_new_element;


        self.add_element(self.next_element_id);
        self.next_element_id +=1; // Increment for the next call
        current_new_element // Return the ID that was actually added
    }


    // Removes an element from the set.
    // If the element is part of a subset, it's removed from that subset.
    // If the subset becomes empty after removal, the subset itself is removed.
    pub fn remove_element(&mut self, element: usize) {
        if let Some(subset_idx) = self.element_to_subset_index.remove(&element) {
            if let Some(set) = self.subsets.get_mut(subset_idx) {
                set.remove(&element);
                if set.is_empty() {
                    // Remove the empty set and adjust indices.
                    // This is tricky because indices in `element_to_subset_index` need updating.
                    // A simpler approach for this version is to mark it for removal or handle during list_subsets,
                    // but tests expect immediate removal and count update.
                    // For now, let's rebuild `subsets` and `element_to_subset_index` if a set becomes empty.
                    // This is not efficient but ensures correctness for test expectations.
                    // A more performant way would be to swap_remove the empty set and update affected indices.

                    self.subsets.remove(subset_idx);
                    // Re-index all elements that were in subsets after the removed one.
                    for (_el, idx) in self.element_to_subset_index.iter_mut() {
                        if *idx > subset_idx {
                            *idx -= 1;
                        }
                    }
                }
            }
        }
    }

    // Unites the subsets containing element1 and element2.
    // If both elements are already in the same subset, nothing changes.
    // If either element does not exist, this operation has no effect.
    // Otherwise, the subset containing element2 is merged into the subset containing element1.
    pub fn union_elements(&mut self, element1: usize, element2: usize) {
        let idx1_opt = self.element_to_subset_index.get(&element1).copied();
        let idx2_opt = self.element_to_subset_index.get(&element2).copied();

        if let (Some(idx1), Some(idx2)) = (idx1_opt, idx2_opt) {
            if idx1 == idx2 {
                // Already in the same set.
                return;
            }

            // Merge set at idx2 into set at idx1.
            // Ensure idx1 is smaller than idx2 to simplify removal and index updates.
            let (target_idx, source_idx) = if idx1 < idx2 { (idx1, idx2) } else { (idx2, idx1) };

            let source_set_elements: Vec<usize> = self.subsets[source_idx].iter().cloned().collect();
            
            // Add elements from source_set to target_set
            for el in source_set_elements.iter() {
                self.subsets[target_idx].insert(*el);
                // Update mapping for moved elements
                self.element_to_subset_index.insert(*el, target_idx);
            }

            // Remove the source_set (which is now empty or its elements moved)
            self.subsets.remove(source_idx);

            // Update indices for subsets that were after the removed source_idx
            for (_el, current_el_subset_idx) in self.element_to_subset_index.iter_mut() {
                if *current_el_subset_idx > source_idx {
                    *current_el_subset_idx -= 1;
                }
            }
        }
    }

    // Checks if element1 and element2 belong to the same subset.
    // Returns true if they are in the same subset, false otherwise (including if one or both elements don't exist).
    pub fn is_same_set(&self, element1: usize, element2: usize) -> bool {
        if element1 == element2 {
            return self.element_to_subset_index.contains_key(&element1);
        }
        match (self.element_to_subset_index.get(&element1), self.element_to_subset_index.get(&element2)) {
            (Some(idx1), Some(idx2)) => idx1 == idx2,
            _ => false, // One or both elements do not exist.
        }
    }

    // Returns the total number of elements in the set.
    pub fn count_elements(&self) -> usize {
        self.element_to_subset_index.len()
    }

    // Returns a list of all non-empty subsets.
    // Each subset is a HashSet<usize>.
    pub fn list_subsets(&self) -> Vec<HashSet<usize>> {
        self.subsets.iter().filter(|s| !s.is_empty()).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator; // Required for HashSet::from_iter

    // Test default state: all elements are in separate subsets.
    #[test]
    fn test_default_state_after_adding_elements() {
        let mut sp = SetPartition::new();
        sp.add_element(0);
        sp.add_element(1);
        sp.add_element(2);

        assert!(!sp.is_same_set(0, 1), "Elements 0 and 1 should be in different sets by default");
        assert!(!sp.is_same_set(1, 2), "Elements 1 and 2 should be in different sets by default");
        assert!(sp.is_same_set(0, 0), "Element 0 should be in the same set as itself");
        let subsets = sp.list_subsets();
        assert_eq!(subsets.len(), 3, "Should be 3 subsets for 3 elements added separately");
        assert!(subsets.iter().any(|s| s.contains(&0) && s.len() == 1));
        assert!(subsets.iter().any(|s| s.contains(&1) && s.len() == 1));
        assert!(subsets.iter().any(|s| s.contains(&2) && s.len() == 1));
    }

    // Test adding an element: it should belong to a new subset.
    #[test]
    fn test_add_element() {
        let mut sp = SetPartition::new();
        sp.add_element(10);
        assert_eq!(sp.count_elements(), 1, "Should have 1 element after adding one");
        let subsets = sp.list_subsets();
        assert_eq!(subsets.len(), 1, "Should have 1 subset after adding one element");
        assert!(subsets[0].contains(&10), "The subset should contain the added element");

        sp.add_element(20); // Add a second element
        assert_eq!(sp.count_elements(), 2, "Should have 2 elements after adding two");
        assert!(!sp.is_same_set(10, 20), "Newly added elements should be in different sets");
        let subsets_after_second_add = sp.list_subsets();
        assert_eq!(subsets_after_second_add.len(), 2, "Should have 2 subsets after adding two elements separately");
    }

    // Test adding a new element using add_new_element
    #[test]
    fn test_add_new_element() {
        let mut sp = SetPartition::new();
        let e1 = sp.add_new_element(); // e.g., 0
        let e2 = sp.add_new_element(); // e.g., 1
        assert_eq!(sp.count_elements(), 2);
        assert_ne!(e1, e2, "Newly generated elements should be distinct");
        assert!(!sp.is_same_set(e1, e2), "New elements from add_new_element should be in different sets");
        let subsets = sp.list_subsets();
        assert_eq!(subsets.len(), 2);
    }

    // Test removing an element.
    #[test]
    fn test_remove_element() {
        let mut sp = SetPartition::new();
        sp.add_element(5);
        sp.add_element(6);
        sp.union_elements(5, 6); // {5, 6}
        sp.add_element(7);       // (7)

        assert_eq!(sp.count_elements(), 3);
        assert!(sp.is_same_set(5, 6));
        assert!(!sp.is_same_set(5, 7));

        sp.remove_element(5); // (6), (7)
        assert_eq!(sp.count_elements(), 2, "Element count should decrease after removal of 5");
        assert!(!sp.element_to_subset_index.contains_key(&5), "Element 5 should be removed from mapping");
        assert!(!sp.is_same_set(5,6), "5 should not be in any set");
        
        let current_subsets = sp.list_subsets();
        assert!(current_subsets.iter().all(|s| !s.contains(&5)), "Element 5 should not be in any subset after removal");
        assert_eq!(current_subsets.len(), 2, "Should be two subsets left: (6) and (7)");
        assert!(current_subsets.contains(&HashSet::from_iter(vec![6])));
        assert!(current_subsets.contains(&HashSet::from_iter(vec![7])));


        sp.remove_element(7); // (6)
        assert_eq!(sp.count_elements(), 1, "Element count should be 1 after removing 7");
        assert!(!sp.element_to_subset_index.contains_key(&7));
        let subsets_after_7_removed = sp.list_subsets();
        assert_eq!(subsets_after_7_removed.len(), 1, "Should be one subset left: (6)");
        assert!(subsets_after_7_removed.contains(&HashSet::from_iter(vec![6])));


        sp.remove_element(6); // {}
        assert_eq!(sp.count_elements(), 0, "Element count should be 0 after removing 6");
        assert!(sp.list_subsets().is_empty(), "All subsets should be gone");
    }

    // Test uniting elements: they should belong to the same subset.
    #[test]
    fn test_union_elements() {
        let mut sp = SetPartition::new();
        sp.add_element(100);
        sp.add_element(101);
        sp.add_element(102);

        assert!(!sp.is_same_set(100, 101));
        sp.union_elements(100, 101); // {100, 101}, {102}
        assert!(sp.is_same_set(100, 101), "Elements 100 and 101 should be in the same set after union");
        assert!(!sp.is_same_set(100, 102), "Elements 100 and 102 should still be in different sets");
        assert_eq!(sp.list_subsets().len(), 2, "Should be 2 subsets after first union");

        sp.union_elements(101, 102); // Transitive: {100, 101, 102}
        assert!(sp.is_same_set(100, 102), "Elements 100 and 102 should be in the same set due to transitivity");
        assert!(sp.is_same_set(100, 101));
        assert!(sp.is_same_set(101, 102));
        assert_eq!(sp.list_subsets().len(), 1, "All elements should now be in a single subset");
        assert_eq!(sp.count_elements(), 3);
        assert_eq!(sp.list_subsets()[0], HashSet::from_iter(vec![100, 101, 102]));
    }

    // Test checking if elements are in the same set.
    #[test]
    fn test_is_same_set() {
        let mut sp = SetPartition::new();
        sp.add_element(0);
        sp.add_element(1);
        sp.add_element(2);

        assert!(sp.is_same_set(0, 0), "Element 0 should be in the same set as itself");
        assert!(!sp.is_same_set(0, 1), "Elements 0 and 1 should initially be in different sets");

        sp.union_elements(0, 1);
        assert!(sp.is_same_set(0, 1), "Elements 0 and 1 should be in the same set after union");
        assert!(sp.is_same_set(1, 0), "is_same_set should be symmetric");
        assert!(!sp.is_same_set(0, 2), "Element 0 and 2 should still be in different sets");

        assert!(!sp.is_same_set(0, 99), "Checking with a non-existent element (99) should be false");
        assert!(!sp.is_same_set(98, 99), "Checking two non-existent elements should be false");
        assert!(!sp.is_same_set(99, 0), "Checking with a non-existent element (99) first should be false");
    }

    // Test counting the number of elements.
    #[test]
    fn test_count_elements() {
        let mut sp = SetPartition::new();
        assert_eq!(sp.count_elements(), 0, "Initially, element count should be 0");

        sp.add_element(10);
        assert_eq!(sp.count_elements(), 1, "Element count should be 1 after adding one element");

        sp.add_element(20);
        assert_eq!(sp.count_elements(), 2, "Element count should be 2 after adding two elements");

        sp.union_elements(10, 20);
        assert_eq!(sp.count_elements(), 2, "Union operation should not change the total element count");

        sp.remove_element(10);
        assert_eq!(sp.count_elements(), 1, "Element count should decrease after removal");

        sp.remove_element(20);
        assert_eq!(sp.count_elements(), 0, "Element count should be 0 after removing all elements");
    }

    // Test listing all subsets.
    #[test]
    fn test_list_subsets() {
        let mut sp = SetPartition::new();
        assert!(sp.list_subsets().is_empty(), "Initially, there should be no subsets");

        sp.add_element(1); // (1)
        sp.add_element(2); // (1), (2)
        let subsets1 = sp.list_subsets();
        assert_eq!(subsets1.len(), 2, "Should be two subsets for two separately added elements");
        assert!(subsets1.contains(&HashSet::from_iter(vec![1])));
        assert!(subsets1.contains(&HashSet::from_iter(vec![2])));


        sp.add_element(3); // (1), (2), (3)
        sp.union_elements(1, 2); // {1, 2}, (3)
        let subsets2 = sp.list_subsets();
        assert_eq!(subsets2.len(), 2, "Should be two subsets after unioning 1 and 2, and adding 3");
        let expected_set12 = HashSet::from_iter(vec![1, 2]);
        let expected_set3 = HashSet::from_iter(vec![3]);
        assert!(subsets2.contains(&expected_set12), "Subsets should contain (1, 2)");
        assert!(subsets2.contains(&expected_set3), "Subsets should contain (3)");


        sp.union_elements(2, 3); // Now all {1, 2, 3} should be in one set
        let subsets3 = sp.list_subsets();
        assert_eq!(subsets3.len(), 1, "Should be one subset after unioning all elements");
        assert_eq!(subsets3[0], HashSet::from_iter(vec![1, 2, 3]), "The single subset should contain (1, 2, 3)");

        sp.remove_element(2); // Remove 2 from {1,2,3} -> {1,3}
        let subsets4 = sp.list_subsets();
        assert_eq!(subsets4.len(), 1, "Should still be one subset");
        assert_eq!(subsets4[0], HashSet::from_iter(vec![1, 3]), "The subset should now be (1, 3)");

        sp.add_element(4); // Add 4, new set (4). So, {1,3}, (4)
        let subsets5 = sp.list_subsets();
        assert_eq!(subsets5.len(), 2, "Should be two subsets: (1,3) and (4)");
        assert!(subsets5.contains(&HashSet::from_iter(vec![1,3])));
        assert!(subsets5.contains(&HashSet::from_iter(vec![4])));
    }
}
