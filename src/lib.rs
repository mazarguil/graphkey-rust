use petgraph::{graph::Graph, Undirected};
use crate::coloring::{Colouring, Kdim};

pub mod coloring;


//
// GraphKey object
//

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct GraphKey(Vec<usize>);

impl GraphKey {
    pub fn new(g : &Graph::<usize, (), Undirected>) -> GraphKey {

        // 1. Generate first colouring & first refine
        let mut gc = Colouring::new(g);
        gc.refine(g);

        // 1.1 If gc is discrete, compute key
        if gc.is_discrete() {
            println!("Already leaf !");
            let descr = gc.compute_graph_from_discrete(g);
            return GraphKey(compute_descriptor(&descr));
        }


        //
        // 2. Create tree root
        //

        let root = {

            let target = gc.select_cell_v1();
            let mut children = gc.get_cell_members(target);
            children.sort_by(|a, b| b.cmp(a));

            TreeNode{
                c : gc,
                target_cell: target,
                children : children,
                son_in_exp_path: None,
                son_k_dim : None,
            }
        };

        //
        // 3. Main loop
        //
        //      * Follows the exploration path of Traces
        //

        //
        // Vars definition
        //
        
        let mut next_list = Vec::from([root]);      // list of colourings to study on next level

        // let mut niter = 0;

        let mut leaf_found = false;

        //
        // Leaf storages
        //

        // let mut leaves_colouing : Vec<Graph<usize, ()>> = Vec::new();
        // let mut leaves_descriptors : Vec<Vec<usize>> = Vec::new();

        //
        //
        //

        while !leaf_found { 

            /* 
            println!();
            println!();
            println!();
            println!("//////////////////////////////");
            println!("//////// Iteration {niter} //////");
            println!("//////////////////////////////");
            niter += 1;
            println!("next_list.len() = {}", next_list.len());
            */

            let current_list = next_list;
            next_list = Vec::new();

            let mut best_k_dim = Kdim::new(0, vec![]);

            for node in current_list.into_iter() {

                /* 
                println!("Working on a new Tree node !");
                println!("Its colouring is :");
                node.c.print_cells();
                println!();
                */

                let mut node = node;

                //
                // 3.1 Add son in exploration to next_list (losing ownership)
                //

                if let Some(b) = node.son_in_exp_path {

                    let k_dim = node.son_k_dim.as_ref().unwrap();

                    if b.c.is_discrete() { leaf_found = true; }

                    if best_k_dim <= *k_dim { 
                        
                        if best_k_dim < *k_dim {
                            next_list = Vec::new();
                            best_k_dim = k_dim.clone();
                        } 
                        
                        next_list.push(*b);
                        
                    }

                    node.son_in_exp_path = None;             
                }

                //
                // 3.2 breath search
                //
                
                while node.children.len() > 0 {

                    //
                    // Create new TreeNode from the individualization of a (graph) node from the target cell
                    //

                    let _v = node.children.pop().unwrap();                       // Pop a random child to individualize
                    let mut _gc = node.c.clone();                            // Clone the colouring (required since it can be edited in each child of a node)
                    let new_color = _gc.individualize(node.target_cell, _v);             // Individualize the selected cell
                    let mut trace = _gc.refine(&g);                         // Refine & generate trace in the process
                    trace.insert(0, new_color);
                    let mut k_dim = Kdim::new(_gc.get_cell_count(), trace);       // Generate Kdim object
                    let mut ancestor_in_exp_path = &mut node;            // parent of the current node "experimental_path_node", created in loop
                                                                                        // at each iteration, the ownership of the current node is given to the parent

                    // println!("This node child {} (indiv = {}) has a Kdim {:?}", child_idx, _v, k_dim);
                    // child_idx += 1;

                    //
                    // Check if current node is k_dim better
                    //

                    if best_k_dim > k_dim {
                        continue;
                    }

                    if best_k_dim < k_dim {
                        next_list = Vec::new();
                        best_k_dim = k_dim.clone();
                    }

                    //
                    // 3.2.1 Compute experimental path
                    //

                    loop {
                        
                        if _gc.is_discrete() {

                            // 1. Compute associated graph
                            // 2. Compute associated Kdim
                            
                            // TODO : check automorphisms


                            let leaf = TreeNode{ 
                                c : _gc, 
                                target_cell: 0,
                                children : vec![],
                                son_in_exp_path: None, 
                                son_k_dim : Some(k_dim)
                            };

                            ancestor_in_exp_path.son_in_exp_path = Some(Box::new(leaf));

                            break;
                        }
                        
                        let target = _gc.select_cell_v1();
                        let mut children = _gc.get_cell_members(target);
                        children.sort_by(|a, b| b.cmp(a));                                                      // TODO : delete
                        let mut new_experimental_path_node = TreeNode{ 
                            c : _gc, target_cell: 
                            target, children : children, 
                            son_in_exp_path: None, 
                            son_k_dim : Some(k_dim)
                        };

                        let _v = new_experimental_path_node.children.pop().unwrap();
                        _gc = new_experimental_path_node.c.clone();
                        let new_color = _gc.individualize(new_experimental_path_node.target_cell, _v);
                        let mut trace = _gc.refine(&g);
                        trace.insert(0, new_color);
                        k_dim = Kdim::new(_gc.get_cell_count(), trace);

                        // Give ownership of the new node to its parent & create a new &mut 

                        ancestor_in_exp_path.son_in_exp_path = Some(Box::new(new_experimental_path_node));
                        ancestor_in_exp_path = ancestor_in_exp_path.son_in_exp_path.as_deref_mut().unwrap();
                    }
                
                    //
                    // 3.2.2 
                    //
                    
                    if let Some(_n) = node.son_in_exp_path {
                        if _n.c.is_discrete() { leaf_found = true; }
                        next_list.push(*_n);
                        node.son_in_exp_path = None;
                    }
                }
            }

            if next_list.len() == 0 {
                panic!("Should not occur !")
            }

        }

        // println!("Final leaf count = {}", next_list.len());

        let canonical = next_list[0].c.compute_graph_from_discrete(&g);
        let mut best_descriptor = compute_descriptor(&canonical);

        for leaf in next_list.into_iter().skip(1) {
            let _canonical = leaf.c.compute_graph_from_discrete(&g);
            let _descriptor = compute_descriptor(&_canonical);
            if _descriptor > best_descriptor {
                best_descriptor = _descriptor;
            }
        }

        return GraphKey(best_descriptor);
    }
}



struct TreeNode {
    c : Colouring,
    target_cell : usize, 
    children : Vec<usize>,
    son_in_exp_path : Option<Box<TreeNode>>,
    son_k_dim : Option<Kdim>,
}














pub fn compute_descriptor(g : &Graph<usize, (), Undirected>) -> Vec<usize> {
    let n = g.node_count();
    let mut cononical = vec![n];
    let mut prev_neigh;

    for i in 0..(n-1)  {
        prev_neigh = i;
        let mut ordered_neighbors : Vec<usize>  = g.neighbors((i as u32).into()).filter(|j| { j.index() > i }).map(|j| { j.index() } ).collect();
        ordered_neighbors.sort(); 
        for j in ordered_neighbors {
            cononical.push(j - prev_neigh);
            prev_neigh = j;
        }
        cononical.push(n);
    }
    
    return cononical
}

//
//
//

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
