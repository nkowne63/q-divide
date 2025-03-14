/// The UnionFind data structure allows dynamic addition of elements and achieves fast find and union operations through path compression and ranked union.
#[derive(Debug)]
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    deleted: Vec<bool>,
}

impl UnionFind {
    /// Creates a UnionFind initialized with n elements.
    /// Each element has its own parent.
    pub fn new(n: usize) -> Self {
        let mut parent = Vec::with_capacity(n);
        let mut rank = Vec::with_capacity(n);
        let mut deleted = Vec::with_capacity(n);
        for i in 0..n {
            parent.push(i);
            rank.push(0);
            deleted.push(false);
        }
        UnionFind { parent, rank, deleted }
    }

    /// Adds a new element and returns the ID of that element.
    pub fn add(&mut self) -> usize {
        let id = self.parent.len();
        self.parent.push(id);
        self.rank.push(0);
        self.deleted.push(false);
        id
    }

    /// Adds multiple elements at once and returns the IDs of each element.
    pub fn add_many(&mut self, count: usize) -> Vec<usize> {
        let start = self.parent.len();
        self.parent.reserve(count);
        self.rank.reserve(count);
        self.deleted.reserve(count);
        let mut ids = Vec::with_capacity(count);
        for i in 0..count {
            let id = start + i;
            self.parent.push(id);
            self.rank.push(0);
            self.deleted.push(false);
            ids.push(id);
        }
        ids
    }

    /// Ensures that the specified number of elements exist.
    /// If the current number of elements is less than n, the missing elements are added.
    pub fn ensure(&mut self, n: usize) {
        let current = self.parent.len();
        if current < n {
            self.add_many(n - current);
        }
    }

    /// Returns the representative element of the set to which element x belongs. (with path compression)
    /// TODO: to be immutable
    pub fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    /// Integrates the sets to which elements x and y belong.
    ///
    /// Returns true if the integration was performed, and false if they already belong to the same set.
    pub fn union(&mut self, x: usize, y: usize) -> bool {
        let x_root = self.find(x);
        let y_root = self.find(y);

        if x_root == y_root {
            return false;
        }

        // Union by rank
        if self.rank[x_root] < self.rank[y_root] {
            self.parent[x_root] = y_root;
        } else if self.rank[x_root] > self.rank[y_root] {
            self.parent[y_root] = x_root;
        } else {
            self.parent[y_root] = x_root;
            self.rank[x_root] += 1;
        }
        true
    }

    /// Determines whether two elements belong to the same set.
    pub fn same_set(&mut self, x: usize, y: usize) -> bool {
        self.find(x) == self.find(y)
    }

    /// Returns the current number of elements.
    pub fn len(&self) -> usize {
        self.deleted.iter().filter(|&&d| !d).count()
    }

    /// Removes an element from the UnionFind.
    /// TODO: shrink the size by change of data structure
    pub fn remove(&mut self, x: usize) {
        if x < self.parent.len() {
            self.parent[x] = x;
            self.rank[x] = 0;
        }
        self.deleted[x] = true;
    }

    /// Returns the number of disjoint sets.
    pub fn count(&mut self) -> usize {
        let mut roots = std::collections::HashSet::new();
        for i in 0..self.parent.len() {
            if self.deleted[i] {
                continue;
            }
            roots.insert(self.find(i));
        }
        roots.len()
    }

    /// Returns the size of the component containing element x.
    pub fn component_size(&mut self, x: usize) -> usize {
        let root = self.find(x);
        let mut size = 0;
        for i in 0..self.parent.len() {
            if self.find(i) == root && !self.deleted[i] {
                size += 1;
            }
        }
        size
    }

    pub fn get_groups(&mut self) -> std::collections::HashMap<usize, Vec<usize>> {
        let mut groups = std::collections::HashMap::new();
        for i in 0..self.parent.len() {
            if self.deleted[i] {
                continue;
            }
            let root = self.find(i);
            groups.entry(root).or_insert(Vec::new()).push(i);
        }
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_and_len() {
        let mut uf = UnionFind::new(5);
        assert_eq!(uf.len(), 5);
        for i in 0..5 {
            assert_eq!(uf.find(i), i);
        }
    }

    #[test]
    fn test_add() {
        let mut uf = UnionFind::new(3);
        let new_id = uf.add();
        assert_eq!(new_id, 3);
        assert_eq!(uf.len(), 4);
        assert_eq!(uf.find(new_id), new_id);
    }

    #[test]
    fn test_add_many() {
        let mut uf = UnionFind::new(2);
        let ids = uf.add_many(3);
        assert_eq!(ids, vec![2, 3, 4]);
        assert_eq!(uf.len(), 5);
        for id in ids {
            assert_eq!(uf.find(id), id);
        }
    }

    #[test]
    fn test_ensure() {
        let mut uf = UnionFind::new(2);
        uf.ensure(5);
        assert_eq!(uf.len(), 5);
        uf.ensure(3);
        assert_eq!(uf.len(), 5);
    }

    #[test]
    fn test_union_and_same_set() {
        let mut uf = UnionFind::new(4);
        assert!(uf.union(0, 1));
        assert!(uf.same_set(0, 1));
        assert!(!uf.union(0, 1));
        assert!(uf.union(2, 3));
        assert!(uf.same_set(2, 3));
        assert!(!uf.same_set(0, 2));
        assert!(uf.union(1, 2));
        for i in 0..4 {
            assert!(uf.same_set(0, i));
        }
    }

    #[test]
    fn test_count() {
        let mut uf = UnionFind::new(5);
        assert_eq!(uf.count(), 5);
        uf.union(0, 1);
        uf.union(1, 2);
        assert_eq!(uf.count(), 3);
    }

    #[test]
    fn test_component_size() {
        let mut uf = UnionFind::new(5);
        for i in 0..5 {
            assert_eq!(uf.component_size(i), 1);
        }
        uf.union(0, 1);
        uf.union(1, 2);
        assert_eq!(uf.component_size(0), 3);
        assert_eq!(uf.component_size(3), 1);
    }

    #[test]
    fn test_remove() {
        let mut uf = UnionFind::new(4);
        uf.union(0, 1);
        uf.remove(1);
        assert_eq!(uf.find(1), 1);
        assert!(!uf.same_set(0, 1));
    }
}