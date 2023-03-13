use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BinaryHeap;
use std::cmp::Reverse;

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use petgraph::Undirected;
use petgraph::graph::{NodeIndex, UnGraph, Graph};

use std::cmp::Ordering;

/// A `Color` is a subset of graph nodes.
///
/// Example : Cell{ color : 0, members : { 0, 1, 2 } }

#[derive(Debug)]
struct Cell {
    color : usize,
    members : HashSet<usize>,
}

/// A `Colouring` is a set of colors covering the graph.
///
/// It is used through the algorithm to characterize the set of distincts nodes
/// More precisely, two nodes u and v are of the same color if they cannot be
/// distinguished by their properties (e.g. neighbor count) and by the
/// hypothesis made so far.
/// 
/// At the start of the algorithm, all nodes are colored with the same color.
/// 
/// size : node count
/// cells[k] : k-th Cell object
/// color_cell[c] : pointer to the cell of color c
/// node_cell[n] : pointer to the cell of the node n
/// node_color[n] = color of the node n
pub struct Colouring {
    size : usize,
    cells : Vec<Rc<RefCell<Cell>>>,                         
    color_cell : HashMap<usize, Rc<RefCell<Cell>>>, 
    node_cell : Vec<Rc<RefCell<Cell>>>,
    node_color : Vec<usize>,
}

impl Colouring {
    
    /// Create ne new uniform colouring of a graph.
    pub fn new<N, E>(g : &UnGraph<N, E>) -> Colouring {

        let size = g.node_count();
        let cell_0 = Cell { color: 0, members : HashSet::from_iter(0..size) };
        let p = Rc::new(RefCell::new(cell_0));

        Colouring {
            size : size,
            cells: vec![ Rc::clone(&p) ],
            color_cell: HashMap::from([ (0, Rc::clone(&p) ) ]),
            node_cell : vec![ Rc::clone(&p) ; size ],
            node_color : vec![ 0; size ],
        }
    }

    /// Clone the given colouring
    /// 
    /// The complete clone of a colouring is required along the exploration of
    ///   
    pub fn clone(&self) -> Colouring {

        let mut new_cells = Vec::new();
        let mut new_color_cell = HashMap::new();
        let mut new_node_cell_map = HashMap::new();
        let mut new_node_color = vec![0;self.size];

        for i in 0..self.cells.len() {

            let _c = (*self.cells[i]).borrow();
            let new_cell = Cell{ color: _c.color, members: _c.members.clone() };
            let new_cell = Rc::new(RefCell::new(new_cell));

            new_cells.push(Rc::clone(&new_cell));
            new_color_cell.insert(_c.color, Rc::clone(&new_cell));

            for k in _c.members.iter() {
                new_node_cell_map.insert(*k, Rc::clone(&new_cell));
                new_node_color[*k] = _c.color;
            }
        }

        let mut new_node_cell = Vec::new();

        for i in 0..self.size {
            new_node_cell.push(new_node_cell_map.remove(&i).unwrap());
        }

        return Colouring {
            size: self.size, 
            cells: new_cells, 
            color_cell: new_color_cell,
            node_cell: new_node_cell, 
            node_color: new_node_color
        }
    }

    /// Checks if the colouring is discrete, i.e. each color is associated to
    /// a single node
    pub fn is_discrete(&self) -> bool {
        return self.cells.len() == self.size;
    }

    pub fn get_cell_count(&self) -> usize {
        return self.cells.len();
    }

    /// TODO : delete
    pub fn get_cell_members(&self, idx : usize) -> Vec<usize> {
        return (*self.cells[idx]).borrow().members.iter().map(|x| *x ).collect()
    }

    /// TODO : delete
    pub fn print_cells(&self) {
        for i in 0..self.cells.len() { 
            print!(" ({})-{:?}", self.cells[i].as_ref().borrow().color, self.cells[i].as_ref().borrow().members);
        }
    }

    /// TODO : delete
    pub fn print_cells_debug(&self) {        

        println!("Cells : ");
        for i in 0..self.cells.len() { 
            print!("Cell {} (color = {}): ", i,  self.cells[i].as_ref().borrow().color);
            println!("{:?}", self.cells[i].as_ref().borrow().members);
        }
        println!("");
        
        println!("Cells by colors : ");
        for (k, c) in self.color_cell.iter() {
            println!("Cell of color {} (color = {}): ", k,  c.as_ref().borrow().color);
        }
        println!("{:?}", self.node_color);
        println!("");

        println!("Node colors : ");
        println!("{:?}", self.node_color);
        println!("");

        println!("Node cells : ");
        for (i, c) in self.node_cell.iter().enumerate() {
            println!("Node {} : color {}", i, c.as_ref().borrow().color);
        }

        println!("");
        println!("");
        
    }

    /// Returns the index of the cell of color c.
    /// 
    /// This index is difficult to keep updated during the optimization, as
    /// splitting a cell c increases all the indexes of cells above c. 
    /// Moreover, the cell index in only needed for the target cell selection
    /// method. Therefore, it seems better to compute the index of the cell in
    /// O(log(N)) time.
    pub fn get_color_index(&self, c : usize) -> usize {
        
        let (mut i, mut j) = (0, self.cells.len());
        if self.cells[0].deref().borrow().color == c { return 0 }
        
        loop {
            let k = (i + j) / 2;
            let _c = self.cells[k].deref().borrow().color;
            if  _c == c { return k }
            else if _c > c { j = k; }
            else { i = k; }
        }
    }

    /// Individualize the node n in the cell of index cell_idx
    /// 
    /// Returns the color of the newly created cell
    pub fn individualize(&mut self, cell_idx : usize, node : usize) -> usize {
        
        // check if the len of the cell is > 1
        assert!(1 < (*self.cells[cell_idx]).borrow().members.len());

        let old_color = (*self.cells[cell_idx]).borrow().color;
        let new_cell = Rc::new(RefCell::new(Cell{ 
            color : old_color, 
            members : HashSet::from([node])
        }));

        // Edit the old cell
        {
            let mut old_cell = (*self.cells[cell_idx]).borrow_mut();
            old_cell.members.remove(&node);
            old_cell.color = old_color+1;
            for u in old_cell.members.iter() {
                self.node_color[*u] = old_color + 1;
            }
        }
        
        // Edit self.cells
        self.cells.insert(cell_idx, Rc::clone(&new_cell));

        // Edit self.cell_color
        if let Some(v) = self.color_cell.remove(&old_color) {
            self.color_cell.insert(old_color+1, v);
        }
        self.color_cell.insert(old_color, Rc::clone(&new_cell));

        // Edit self.node_cell
        self.node_cell[node] = Rc::clone(&new_cell);

        return old_color+1;

    }

    /// Split the cell into two cells, such that the first one contains
    /// the nodes in new_members
    pub fn split_cell(&mut self, cell_idx : usize, new_members : Vec<usize>) {
        
        let old_color = (*self.cells[cell_idx]).borrow().color;
        let new_color = old_color + new_members.len();

        // Generate the new cell
        let new_cell = Rc::new(RefCell::new(Cell{ 
            color : old_color, 
            members : HashSet::from_iter(new_members.clone())
        }));

        // Edit the old cell
        {
            let mut old_cell = (*self.cells[cell_idx]).borrow_mut();

            for u in new_members.iter() {
                old_cell.members.remove(&u);
            }
            let new_color = old_cell.color + new_members.len();
            old_cell.color = new_color; 

            for u in old_cell.members.iter() {
                self.node_color[*u] = new_color;
            }
        }

        // Edit self.cells
        self.cells.insert(cell_idx, Rc::clone(&new_cell));

        // Edit self.cell_color
        if let Some(v) = self.color_cell.remove(&old_color) {
            self.color_cell.insert(new_color, v);
        }
        self.color_cell.insert(old_color, Rc::clone(&new_cell));

        // Edit self.node_cell
        for u in new_members {
            self.node_cell[u] = Rc::clone(&new_cell);
        }

    }

    /// Refine a Colouring according to the graph g.
    /// 
    /// This function is implemented in an isomorhpic-invariant way, i.e. for
    /// any permutation P of [1..N], any graph G and any coulouring C, we have 
    /// P(C).refine( P(G) ) == P( C.refine(G) )
    /// 
    /// For more deatails, see https://doi.org/10.1016/j.jsc.2013.09.003
    /// 
    pub fn refine<N, E>(&mut self, g : &UnGraph<N, E>) -> Vec<usize>
    {
    
    if self.is_discrete() {
        return vec![];
    }

        let mut trace = Vec::new();

        // Uncounted_colors = set of colors to handle, updated during the main loop.
        // A heap is used so that the colors are explored in a deterministic order.
        // The elements in the heap are in reversed order in order to minimize the Trace
        // TODO : For now, all cells are added. Later, start only with the newly generated color, passed as argument
        // CANDO : benchmark with non-reversed elements
        let mut uncounted_colors = BinaryHeap::new();
        for (k, _) in self.color_cell.iter() {
            uncounted_colors.push(Reverse(*k));
        }

        loop {
            
            let studied_color = uncounted_colors.pop();

            // break condition            
            if studied_color == None { break; }
            let Reverse(studied_color) = studied_color.unwrap();
            
            // remove potential duplicates
            loop {
                if let Some(_next) = uncounted_colors.peek() {
                    if _next.0 == studied_color {
                        uncounted_colors.pop();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } 

            // degrees[n] = # of connections between node n and studied_cell
            // visited_cells keeps the set of cells visited while iteration 
            let mut degrees : HashMap<usize, usize> = HashMap::new();       
            let mut visited_cells : HashSet<usize> = HashSet::new();        

            // Fill the degree map
            // In brackets in order to drom the Cell after iteration
            {
                let studied_cell = self.color_cell.get(&studied_color).unwrap().deref().borrow();
                for u in studied_cell.members.iter() {
                    for v in g.neighbors( NodeIndex::new(*u) ) {
                        degrees.entry(v.index()).and_modify(|counter| *counter += 1).or_insert(1);
                        visited_cells.insert(self.node_color[v.index()]);
                    }
                }
            }
            
            // For each visited cell (iter in order of color)
            let mut visited_cells : Vec<usize> = visited_cells.into_iter().collect();
            visited_cells.sort();

            for _color in visited_cells {
                
                // Do not process if cell is singleton                
                if (*(*self.color_cell.get(&_color).unwrap())).borrow().members.len() == 1 {
                    continue;
                }

                let mut _cell_idx = self.get_color_index(_color);

                // Get cell subset according to degree

                let c1 = self.color_cell.get(&_color).unwrap();
                let mut splits : HashMap<usize, Vec<usize>> = HashMap::new();

                for u in c1.deref().borrow().members.iter() {
                    
                    let _d = match degrees.get(u) {
                        None => { 0 },
                        Some(n) => { *n }
                    };

                    if let Some(m) = splits.get_mut(&_d) { 
                        m.push(*u);
                    } else {
                        splits.insert(_d, vec![*u] );
                    }
                }

                // Do not split the cell if no degree difference
                if splits.len() == 1 { continue; }

                // Get the list of different degrees                
                let mut splits_degrees : Vec<usize> = Vec::with_capacity(splits.len());
                for (_d, _) in splits.iter() { splits_degrees.push(*_d); }
                splits_degrees.sort();
                let last_degree = splits_degrees.pop().unwrap();

                // Split cell according to degree (splits are made with increasing degrees)
                for _d in splits_degrees {
    
                    // Split cell
                    let h = splits.remove(&_d).unwrap();
                    // let h_len = h.len();
                    self.split_cell(_cell_idx, h);
                    
                    // Add new cell to uncounted
                    let new_c = (*self.cells[_cell_idx]).borrow().color;
                    uncounted_colors.push(Reverse(new_c));    
                    
                    // update trace
                    trace.push((*self.cells[_cell_idx+1]).borrow().color);

                    // update cell index 
                    _cell_idx += 1;
                }

                // Add the last cell to uncounted
                {
                    let h = splits.remove(&last_degree).unwrap();
                    if h.len() > 1 {
                        let new_c = (*self.cells[_cell_idx]).borrow().color;
                        uncounted_colors.push(Reverse(new_c));    
                    }
                }
            } 
        }

        return trace;
    }

    //
    // Cell selection
    // TODO

    pub fn select_cell_v1(&self) -> usize {
        
        for i in 0..self.cells.len() {
            if (*self.cells[i]).borrow().members.len() > 1 {
                return i;
            }
        }
        
        panic!("select_cell called on a discrete coloring");
    }

    /// Generate the desriptor associated to the colouring
    pub fn compute_graph_from_discrete(&self, g : &Graph<usize, (), Undirected>) -> Graph<usize, (), Undirected> {

        assert!(self.is_discrete());

        let edges : Vec<(usize, usize)> = g.edge_indices()
            .map(|e| { 
                let (u, v) = g.edge_endpoints(e).unwrap();
                (self.node_color[u.index()] , self.node_color[v.index()])
            })
            .collect();
        
        let mut g = UnGraph::<usize, ()>::new_undirected();
        
        g.reserve_nodes(self.size);
        (0..self.size).for_each(|_| { g.add_node(1); });
        
        g.reserve_edges(edges.len());
        edges.into_iter().for_each(|(u, v)| { g.add_edge(NodeIndex::new(u), NodeIndex::new(v), ()); });
        
        return g
    }
}










 


/// K-dim coloring (see TODO)
#[derive(Debug, Eq, Clone)]
pub struct Kdim (usize, Vec<usize>);

impl Kdim {
    pub fn new(u : usize, v : Vec<usize>) -> Kdim {
        Kdim(u, v)
    }
}


impl Ord for Kdim {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0 > other.0 { return Ordering::Greater; }
        if self.0 < other.0 { return Ordering::Less; }
        return self.1.cmp(&other.1).reverse();
    }
}

impl PartialOrd for Kdim {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Kdim {
    fn eq(&self, other: &Self) -> bool {
        if self.0 != other.0 {
            return false
        }
        return self.1 == other.1;
    }
}






