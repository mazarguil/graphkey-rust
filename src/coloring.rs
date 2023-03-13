use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BinaryHeap;
use std::cmp::Reverse;

use petgraph::Undirected;
use petgraph::graph::{NodeIndex, UnGraph, Graph};

use std::cmp::Ordering;

/// A `Color` is a subset of graph nodes.
///
/// Example : Cell{ color : 0, members : { 0, 1, 2 } }

#[derive(Debug, Clone)]
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
/// size : node count of the graph
/// cells[k] : k-th Cell object
/// color_cell[c] : pointer to the cell of color c
/// node_cell[n] : pointer to the cell of the node n
/// node_color[n] = color of the node n
/// 

#[derive(Clone)]
pub struct Colouring {
    size : usize,
    cells : Vec<Cell>,
    color_cell : HashMap<usize, usize>, 
    node_cell : Vec<usize>,
    node_color : Vec<usize>,
}

impl Colouring {
    
    /// Create ne new uniform colouring of a graph.
    pub fn new<N, E>(g : &UnGraph<N, E>) -> Colouring {

        let size = g.node_count();
        let cell_0 = Cell { color: 0, members : HashSet::from_iter(0..size) };
        
        Colouring {
            size : size,
            cells: vec![ cell_0 ],
            color_cell: HashMap::from([ (0, 0) ]),
            node_cell : vec![ 0 ; size ],
            node_color : vec![ 0; size ],
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
        return self.cells[idx].members.iter().map(|x| *x ).collect()
    }

    /// TODO : delete
    pub fn print_cells(&self) {
        for i in 0..self.cells.len() { 
            print!(" ({})-{:?}", self.cells[i].color, self.cells[i].members);
        }
    }

    /// TODO : delete
    pub fn print_cells_debug(&self) {        

        println!("Cells : ");
        for i in 0..self.cells.len() { 
            print!("Cell {} (color = {}): ", i,  self.cells[i].color);
            println!("{:?}", self.cells[i].members);
        }
        println!("");
        
        println!("Cells by colors : ");
        for (k, c) in self.color_cell.iter() {
            println!("Cell of color {} (color = {}): ", k,  self.cells[*c].color);
        }
        println!("{:?}", self.node_color);
        println!("");

        println!("Node colors : ");
        println!("{:?}", self.node_color);
        println!("");

        println!("Node cells : ");
        for (i, c) in self.node_cell.iter().enumerate() {
            println!("Node {} : color {}", i, self.cells[*c].color);
        }

        println!("");
        println!("");
        
    }

    /// Individualize the node n in the cell of index cell_idx
    /// 
    /// Returns the color of the newly created cell
    pub fn individualize(&mut self, cell_idx : usize, node : usize) -> usize {
        
        // check if the len of the cell is > 1
        assert!(1 < self.cells[cell_idx].members.len());

        let new_cell_index = self.cells.len();

        let old_color = self.cells[cell_idx].color;
        let new_cell = Cell{ 
            color : old_color, 
            members : HashSet::from([node])
        };

        // Edit the old cell
        {
            let mut old_cell = &mut self.cells[cell_idx];
            old_cell.members.remove(&node);
            old_cell.color = old_color+1;
            for u in old_cell.members.iter() {
                self.node_color[*u] = old_color + 1;
            }
        }
        
        // Edit self.cells
        self.cells.push(new_cell);

        // Edit self.color_cell
        if let Some(old_cell_index) = self.color_cell.remove(&old_color) {
            self.color_cell.insert(old_color+1, old_cell_index);
        }
        self.color_cell.insert(old_color, new_cell_index);

        // Edit self.node_cell
        self.node_cell[node] = new_cell_index;

        return old_color+1;

    }

    /// Split the cell into two cells, such that the first one contains
    /// the nodes in new_members
    pub fn split_cell(&mut self, cell_idx : usize, new_members : Vec<usize>) -> usize {
        
        let old_color = self.cells[cell_idx].color;
        let new_color = old_color + new_members.len();
        let new_cell_index = self.cells.len();

        // Generate the new cell
        let new_cell = Cell{ 
            color : old_color, 
            members : HashSet::from_iter(new_members.clone())
        };

        // Edit the old cell
        {
            let mut old_cell = &mut self.cells[cell_idx];

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
        self.cells.push(new_cell);

        // Edit self.cell_color
        if let Some(v) = self.color_cell.remove(&old_color) {
            self.color_cell.insert(new_color, v);
        }
        self.color_cell.insert(old_color, new_cell_index);

        // Edit self.node_cell
        for u in new_members {
            self.node_cell[u] = new_cell_index;
        }

        return new_color
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
                let studied_cell = &self.cells[*self.color_cell.get(&studied_color).unwrap()];
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
                
                let _cell_idx = *self.color_cell.get(&_color).unwrap();

                // Do not process if cell is singleton                
                if self.cells[_cell_idx].members.len() == 1 {
                    continue;
                }

                // Get cell subset according to degree


                let mut splits : HashMap<usize, Vec<usize>> = HashMap::new();

                {
                    let c1 = &self.cells[_cell_idx];
                    
                    for u in c1.members.iter() {
                        
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
                    let new_color = self.split_cell(_cell_idx, h);
                    
                    // Add new cell to uncounted
                    uncounted_colors.push(Reverse(new_color));    
                    
                    // update trace
                    trace.push(new_color);
                }

                // Add the last cell to uncounted
                {
                    let h = splits.remove(&last_degree).unwrap();
                    if h.len() > 1 {
                        let new_c = self.cells[_cell_idx].color;
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
            if self.cells[i].members.len() > 1 {
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






