use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::BinaryHeap;
use std::cmp::Reverse;

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::mem;

use petgraph::Undirected;
use petgraph::data::Build;
use petgraph::graph::{NodeIndex, UnGraph};
use petgraph::stable_graph::IndexType;

//
//
//

#[derive(Debug)]
struct Cell {
    color : usize,
    members : HashSet<usize>,
}

pub struct Colouring {

    size : usize,

    cells : Vec<Rc<RefCell<Cell>>>,                         // cells[k] = set of nodes in the cell k
    color_cell : HashMap<usize, Rc<RefCell<Cell>>>,         // color_cell[c] = cell of color c

    node_cell : Vec<Rc<RefCell<Cell>>>,                     // node_cell[n] = cell of node n
    node_color : Vec<usize>,                                // node_color[n] = color of node n
}

impl Colouring {
    
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

    pub fn clone(&self) -> Colouring {

        let mut new_cells = Vec::new();
        let mut new_color_cell = HashMap::new();

        let mut new_node_cell_map = HashMap::new();
        let mut new_node_color = vec![0;self.size];

        // fill cells
        for i in 0..self.cells.len() {

            let _c = (*self.cells[i]).borrow();
            let new_cell = Cell{
                color: _c.color,
                members: _c.members.clone()
            };
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

        let rep = Colouring {
            size: self.size, 
            cells: new_cells, 
            color_cell: new_color_cell,
            node_cell: new_node_cell, 
            node_color: new_node_color
        };

        return rep;

    }

    pub fn is_discrete(&self) -> bool {
        return self.cells.len() == self.size;
    }

    //
    //
    //

    pub fn get_cell_members(&self, idx : usize) -> Vec<usize> {
        return (*self.cells[idx]).borrow().members.iter().map(|x| *x ).collect()
    }

    //
    // print : TODO !
    //

    pub fn print_cells_debug(&self) {
        
        println!("------------------------------");
        println!("------------------------------");
        println!("------------------------------");
        

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


    //
    // return the index of the cell of color c
    //

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

    //
    // node individualization
    //

    pub fn individualize(&mut self, cell_idx : usize, node : usize) -> usize {
        
        //
        // check if the len of the cell is > 1
        //

        assert!(1 < (*self.cells[cell_idx]).borrow().members.len());

        //
        // vars
        //

        let old_color = (*self.cells[cell_idx]).borrow().color;

        //
        // Generate the new cell
        //

        let new_cell = Rc::new(RefCell::new(Cell{ 
            color : old_color, 
            members : HashSet::from([node])
        }));

        // 
        // Edit the old cell
        //

        {
            let mut old_cell = (*self.cells[cell_idx]).borrow_mut();
            old_cell.members.remove(&node);
            old_cell.color = old_color+1;

            //
            // Edit self.node_color
            //

            for u in old_cell.members.iter() {
                self.node_color[*u] = old_color + 1;
            }
        }
        
        //
        // Edit self.cells
        //

        self.cells.insert(cell_idx, Rc::clone(&new_cell));

        //
        // Edit self.cell_color
        //

        if let Some(v) = self.color_cell.remove(&old_color) {
            self.color_cell.insert(old_color+1, v);
        }
        self.color_cell.insert(old_color, Rc::clone(&new_cell));

        //
        // Edit self.node_cell
        //

        self.node_cell[node] = Rc::clone(&new_cell);

        return old_color+1;

    }

    //
    // Split cell
    //


    // returns the color of the newly created cell
    pub fn split_cell(&mut self, cell_idx : usize, new_members : Vec<usize>) {
        
        assert!(new_members.len() > 0);                                                                                                         // check 
        assert!(cell_idx < self.cells.len());
        assert!((*self.cells[cell_idx]).borrow().members.len() > new_members.len());                                                            // check if removing less nodes than the cell
        assert!((*self.cells[cell_idx]).borrow().members.iter().all(|x| { self.node_color[*x] == (*self.cells[cell_idx]).borrow().color }) );   // check if removed nodes are in the cell
        
        //
        // vars
        //

        let old_color = (*self.cells[cell_idx]).borrow().color;
        let new_color = old_color + new_members.len();

        //
        // Generate the new cell
        //

        let new_cell = Rc::new(RefCell::new(Cell{ 
            color : old_color, 
            members : HashSet::from_iter(new_members.clone())
        }));

        // 
        // Edit the old cell
        //

        {
            let mut old_cell = (*self.cells[cell_idx]).borrow_mut();

            for u in new_members.iter() {
                old_cell.members.remove(&u);
            }
            let new_color = old_cell.color + new_members.len();
            old_cell.color = new_color; 

            //
            // Edit self.node_color
            //

            for u in old_cell.members.iter() {
                self.node_color[*u] = new_color;
            }
        }

        //
        // Edit self.cells
        //

        self.cells.insert(cell_idx, Rc::clone(&new_cell));

        //
        // Edit self.cell_color
        //

        if let Some(v) = self.color_cell.remove(&old_color) {
            self.color_cell.insert(new_color, v);
        }
        self.color_cell.insert(old_color, Rc::clone(&new_cell));

        //
        // Edit self.node_cell
        //

        for u in new_members {
            self.node_cell[u] = Rc::clone(&new_cell);
        }

    }

    // 
    // 
    // 

    pub fn refine<N, E>(&mut self, g : &UnGraph<N, E>) -> Vec<usize>
    {

        //
        // 0. Check
        //

        assert!( !self.is_discrete() );

        //
        // 1. Setup vars
        //

        let mut trace = Vec::new();

        //
        // uncounted_colors = set of colors to handle.
        // - For now, all cells are added. Later, the set can be put as parameter
        //

        let mut uncounted_colors = BinaryHeap::new();
        for (k, _) in self.color_cell.iter() {
            uncounted_colors.push(Reverse(*k));
        }

        //
        // 2. loop over the cells to explore
        //

        let mut n_iter = 0; 

        loop {
            
            /* 
            println!();
            println!();
            println!();
            println!();
            println!("//////////////////////////////");
            println!("////// Iteration {} ///////////", n_iter);
            println!("//////////////////////////////");
            self.print_cells();
            */

            
            n_iter = n_iter + 1;

            //
            // TODO : define label-invariant deterministic order for exploring uncounted_colors (doable ?)
            //

            // println!("Remaining stack : {:?}", uncounted_colors);
            let studied_color = uncounted_colors.pop();

            //
            // break condition            
            //
            
            if studied_color == None {
                break;
            } 
            
            let Reverse(studied_color) = studied_color.unwrap();
            let studied_cell = self.color_cell.get(&studied_color).unwrap().deref().borrow();
            
            // println!("Popped color : {studied_color}");

            //
            // Setup vars
            //

            let mut degrees : HashMap<usize, usize> = HashMap::new();       // degrees[n] = #of connections between node n and studied_cell
            let mut visited_cells : HashSet<usize> = HashSet::new();        // Map of cells visited while 

            //
            // fill the degree map
            //

            for u in studied_cell.members.iter() {
                for v in g.neighbors( NodeIndex::new(*u) ) {
                    degrees.entry(v.index()).and_modify(|counter| *counter += 1).or_insert(1);
                    visited_cells.insert(self.node_color[v.index()]);
                }
            }

            drop(studied_cell);
            
            //
            // for each visited cell (in order of coloring !)
            // 

            let mut visited_cells : Vec<usize> = visited_cells.into_iter().collect();
            visited_cells.sort();

            for _color in visited_cells {
                
                //
                // Do not process if cell is singleton
                //
                
                if (*(*self.color_cell.get(&_color).unwrap())).borrow().members.len() == 1 {
                    continue;
                }

                let mut _cell_idx = self.get_color_index(_color);

                //
                // get cell subset according to degree
                //

                let c1 = self.color_cell.get(&_color).unwrap();
                let mut splits : HashMap<usize, Vec<usize>> = HashMap::new();

                // let mut max_subset_size = 0;
                // let mut degree_of_max_subset_size = 0;

                for u in c1.deref().borrow().members.iter() {
                    
                    let _d = match degrees.get(u) {
                        None => { 
                            0 
                        },
                        Some(n) => { *n }
                    };

                    if let Some(m) = splits.get_mut(&_d) { 
                        m.push(*u);
                    } else {
                        splits.insert(_d, vec![*u] );
                    }
                }
                
                // println!("Resulting splits for coll of color {_color} : {:?}", splits);

                //
                // Do not split if no degree difference
                //

                if splits.len() == 1 {
                    continue;
                }

                //
                // Split cell according to degree
                //
                // cell is split with increasing degrees
                // 

                let mut splits_degrees : Vec<usize> = Vec::new();
                for (_d, _) in splits.iter() {
                    splits_degrees.push(*_d);
                }

                splits_degrees.sort();
                let last_degree = splits_degrees.pop().unwrap();

                for _d in splits_degrees {
    
                    //
                    // Split cell
                    //

                    let h = splits.remove(&_d).unwrap();
                    let h_len = h.len();
                    self.split_cell(_cell_idx, h);
                    
                    //
                    // Add new cell to uncounted
                    //

                    let new_c = (*self.cells[_cell_idx]).borrow().color;
                    if h_len > 1 {
                        uncounted_colors.push(Reverse(new_c));    
                    }

                    //
                    // update trace
                    // 

                    trace.push((*self.cells[_cell_idx+1]).borrow().color);

                    //
                    // update cell index
                    //

                    _cell_idx += 1;
                }

                //
                // Add the last cell to uncounted
                //

                {
                    let h = splits.remove(&last_degree).unwrap();
                    if h.len() > 1 {
                        let new_c = (*self.cells[_cell_idx]).borrow().color;
                        uncounted_colors.push(Reverse(new_c));    
                    }
                }
            } 
        }

        // println!("Resulting trace = {:?}", trace);
        return trace;
    }

    //
    // Cell selection
    //

    pub fn select_cell_v1(&self) -> usize {
        
        for i in 0..self.cells.len() {
            if (*self.cells[i]).borrow().members.len() > 1 {
                return i;
            }
        }
        
        panic!("select_cell called on a discrete coloring");
    }

    //
    // Generate graph with canonical labelling
    //

    pub fn compute_canonical<N, E>(&self, g : &UnGraph<N, E>) -> UnGraph<usize, ()> {

        assert!(self.is_discrete());

        let edges : Vec<(usize, usize)> = g.edge_indices()
            .map(|e| { 
                let (u, v) = g.edge_endpoints(e).unwrap();
                (self.node_color[u.index()] , self.node_color[v.index()])
            })
            .collect();
        
        let mut g = UnGraph::<usize, ()>::new_undirected();
        
        g.reserve_nodes(self.size);
        (0..self.size).for_each(|i| { g.add_node(i); });
        
        g.reserve_edges(edges.len());
        edges.into_iter().for_each(|(u, v)| { g.add_edge(NodeIndex::new(u), NodeIndex::new(v), ()); });
        
        return g
    }



}
