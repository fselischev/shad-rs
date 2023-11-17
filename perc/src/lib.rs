#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

use std::{
    collections::{HashSet, VecDeque},
    fmt::Display,
};

use rand::{distributions::Bernoulli, prelude::Distribution};

type Cell = (usize, usize);

/// Represents a grid of boolean values.
pub struct BoolGrid {
    width: usize,
    height: usize,
    data: Vec<Vec<bool>>,
}

impl BoolGrid {
    /// Creates a new grid with all values initialized as `false`.
    ///
    /// # Arguments
    ///
    /// * `width` - grid width.
    /// * `height` - grid height.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![vec![false; height]; width],
        }
    }

    /// Creates a new grid with every value initialized randomly.
    ///
    /// # Arguments
    ///
    /// * `width` - grid width.
    /// * `height` - grid height.
    /// * `vacancy` - probability of any given value being equal
    /// to `false`.
    pub fn random(width: usize, height: usize, vacancy: f64) -> Self {
        Self {
            width,
            height,
            data: {
                let mut data = vec![vec![false; height]; width];
                let d = Bernoulli::new(1.0 - vacancy).expect("given prob should be valid");
                let mut rng = rand::thread_rng();
                for x in 0..width {
                    for y in 0..height {
                        data[x][y] = d.sample(&mut rng);
                    }
                }

                data
            },
        }
    }

    /// Returns grid width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Returns grid height.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Returns the current value of a given cell.
    /// The caller must ensure that `x` and `y` are valid.
    ///
    /// # Arguments
    ///
    /// * `x` - must be >= 0 and < grid width.
    /// * `y` - must be >= 0 and < grid height.
    ///
    /// # Panics
    ///
    /// If `x` or `y` is out of bounds, this method may panic
    /// (or return incorrect result).
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.data[x][y]
    }

    /// Sets a new value to a given cell.
    /// The caller must ensure that `x` and `y` are valid.
    ///
    /// # Arguments
    ///
    /// * `x` - must be >= 0 and < grid width.
    /// * `y` - must be >= 0 and < grid height.
    ///
    /// # Panics
    ///
    /// If `x` or `y` is out of bounds, this method may panic
    /// (or set value to some other unspecified cell).
    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.data[x][y] = value;
    }

    pub fn dfs_roots(&self) -> Vec<Cell> {
        let mut roots = Vec::with_capacity(self.width);
        let mut trunc_cnt = 0;
        for x in 0..self.width {
            for y in 0..self.height {
                if y == 0 && !self.data[x][y] {
                    roots.push((x, y));
                    trunc_cnt += 1;
                }
            }
        }
        roots.truncate(trunc_cnt);
        roots
    }

    pub fn neighbours<'a>(&'a self, x: usize, y: usize) -> impl Iterator<Item = Cell> + 'a {
        [(-1, 0), (0, -1), (0, 1), (1, 0)]
            .iter()
            .filter_map(move |(dx, dy)| {
                let x = x as isize + dx;
                let y = y as isize + dy;

                if x >= 0
                    && x < self.width as isize
                    && y >= 0
                    && y < self.height as isize
                    && !self.data[x as usize][y as usize]
                {
                    return Some((x as usize, y as usize));
                }
                None
            })
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Returns `true` if the given grid percolates. That is, if there is a path
/// from any cell with `y` == 0 to any cell with `y` == `height` - 1.
/// If the grid is empty (`width` == 0 or `height` == 0), it percolates.
pub fn percolates(grid: &BoolGrid) -> bool {
    if grid.height == 0 || grid.width == 0 {
        return true;
    }
    let roots = grid.dfs_roots();
    if roots.is_empty() {
        return false;
    }

    let mut visited = HashSet::new();
    let mut queue = VecDeque::with_capacity(roots.len());
    roots.iter().for_each(|v| queue.push_back(*v));

    while let Some((x, y)) = queue.pop_front() {
        if y == grid.height - 1 {
            return true;
        }

        for nb in grid.neighbours(x, y) {
            if visited.insert(nb) {
                queue.push_front(nb)
            }
        }
    }

    false
}

impl Display for BoolGrid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in 0..self.width {
            for y in 0..self.height {
                if self.data[x][y] {
                    write!(f, "#")?;
                } else {
                    write!(f, ".")?;
                }
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

const N_TRIALS: u64 = 10000;

/// Returns an estimate of the probability that a random grid with given
/// `width, `height` and `vacancy` probability percolates.
/// To compute an estimate, it runs `N_TRIALS` of random experiments,
/// in each creating a random grid and checking if it percolates.
pub fn evaluate_probability(width: usize, height: usize, vacancy: f64) -> f64 {
    let mut perc_count = 0;
    for _ in 0..N_TRIALS {
        let grid = BoolGrid::random(width, height, vacancy);
        if percolates(&grid) {
            perc_count += 1;
        }
    }
    return perc_count as f64 / N_TRIALS as f64;
}
