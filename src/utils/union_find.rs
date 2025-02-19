/// The UnionFind data structure allows dynamic addition of elements and achieves fast find and union operations through path compression and ranked union.
#[derive(Debug)]
pub struct UnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl UnionFind {
    /// Creates a UnionFind initialized with n elements.
    /// Each element has its own parent.
    pub fn new(n: usize) -> Self {
        let mut parent = Vec::with_capacity(n);
        let mut rank = Vec::with_capacity(n);
        for i in 0..n {
            parent.push(i);
            rank.push(0);
        }
        UnionFind { parent, rank }
    }

    /// Adds a new element and returns the ID of that element.
    pub fn add(&mut self) -> usize {
        let id = self.parent.len();
        self.parent.push(id);
        self.rank.push(0);
        id
    }

    /// Adds multiple elements at once and returns the IDs of each element.
    pub fn add_many(&mut self, count: usize) -> Vec<usize> {
        let start = self.parent.len();
        self.parent.reserve(count);
        self.rank.reserve(count);
        let mut ids = Vec::with_capacity(count);
        for i in 0..count {
            let id = start + i;
            self.parent.push(id);
            self.rank.push(0);
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
        self.parent.len()
    }
}