use petgraph::visit::{NodeCompactIndexable, IntoNeighbors, IntoEdges, EdgeRef};
use crate::coloring::{Colouring, Kdim};

pub mod coloring;


//
// GraphKey object
//

#[derive(Debug, Hash, PartialEq, Eq)]
pub struct GraphKey(Vec<usize>);

impl GraphKey {
    pub fn get_descriptor(&self) -> &Vec<usize> {
        return &self.0
    }
}

impl GraphKey {
    pub fn new<G>(g : G) -> GraphKey 
    where
        G : NodeCompactIndexable + IntoNeighbors + IntoEdges
    {

        // Generate first colouring & first refine.
        let mut gc = Colouring::new(g);
        gc.refine(g);

        // If gc is discrete, compute the associated key.
        if gc.is_discrete() {
            let descr = gc.compute_graph_from_discrete(g);
            return GraphKey(compute_descriptor(&descr));
        }

        // Otherwise, set up the tree for exploration.
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

        let mut next_list = Vec::from([root]);      // list of colourings to study on next level
        let mut leaf_found = false;

        // let mut leaves_colouing : Vec<Graph<usize, ()>> = Vec::new();
        // let mut leaves_descriptors : Vec<Vec<usize>> = Vec::new();

        while !leaf_found { 

            let current_list = next_list;
            next_list = Vec::new();

            let mut best_k_dim = Kdim::new(0, vec![]);

            for node in current_list.into_iter() {

                // node.c.print_cells();

                let mut node = node;

                // Add son in exploration to next_list (losing ownership)
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

                while node.children.len() > 0 {

                    // Create new TreeNode from the individualization of a (graph) node from the target cell
                    let _v = node.children.pop().unwrap();
                    let mut _gc = node.c.clone();
                    let new_color = _gc.individualize(node.target_cell, _v);
                    let mut trace = _gc.refine(&g);
                    trace.insert(0, new_color);
                    let mut k_dim = Kdim::new(_gc.get_cell_count(), trace);

                    // at each iteration, the ownership of the current node is given to the parent
                    let mut ancestor_in_exp_path = &mut node;
                    
                    if best_k_dim > k_dim {
                        continue;
                    }

                    if best_k_dim < k_dim {
                        next_list = Vec::new();
                        best_k_dim = k_dim.clone();
                    }

                    // Compute experimental path
                    loop {
                        
                        if _gc.is_discrete() {
                            
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
                        children.sort_by(|a, b| b.cmp(a));             // TODO : delete
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
                    
                    if let Some(_n) = node.son_in_exp_path {
                        if _n.c.is_discrete() { leaf_found = true; }
                        next_list.push(*_n);
                        node.son_in_exp_path = None;
                    }
                }
            }
        }

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

fn compute_descriptor<G>(g : G) -> Vec<usize>
where
    G : NodeCompactIndexable + IntoNeighbors + IntoEdges
{
    let n = g.node_count();
    let mut canonical = vec![n];
    let mut prev_neigh;

    for i in 0..(n-1)  {
        prev_neigh = i;
        let mut ordered_neighbors : Vec<usize>  = g.neighbors(g.from_index(i)).filter(|j| { g.to_index(*j) > i }).map(|j| { g.to_index(j) } ).collect();
        ordered_neighbors.sort(); 
        for j in ordered_neighbors {
            canonical.push(j - prev_neigh);
            prev_neigh = j;
        }
        canonical.push(n);
    }
    
    return canonical
}

//
//
//

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::{NodeIndex, UnGraph};
    use petgraph::{Graph, Undirected};
    use rand::{Rng, thread_rng};
    use rand::seq::SliceRandom;
    use std::collections::HashSet;
    use petgraph::algo::is_isomorphic;

    fn gen_test_graph() -> Graph::<usize, (), Undirected> {
    
        let edges : Vec<(u32, u32)> = vec![
            (0, 3), (0, 5), (0, 8), (1, 4), (1, 6), (1, 8),
            (2, 5), (2, 7), (3, 6), (3, 9), (4, 7), (4, 9),
            (5, 8), (7, 9)
        ];
    
        return UnGraph::from_edges(edges);
    }

    
    fn generate_random_graph(n : usize, p : f64) -> Graph::<usize, (), Undirected> {
        
        let mut rng = rand::thread_rng();
        let mut g = UnGraph::<usize, ()>::new_undirected();
        g.reserve_nodes(n);
        (0..n).for_each(|i| { g.add_node(i); });
        
        for i in 0..n {
            for j in (i+1)..n {
                if rng.gen_range((0.)..(1.)) < p {
                    g.add_edge(NodeIndex::new(i), NodeIndex::new(j), ());
                }
            }
        }

        return g
    }

    
    fn generate_permutated_graph(g : &Graph::<usize, (), Undirected>) -> Graph::<usize, (), Undirected> {

        let n = g.node_count();
        let mut perm : Vec<usize> = (0..n).collect();
        let mut rng = thread_rng();
        perm.shuffle(&mut rng);

        
        let edges : Vec<(usize, usize)> = g.edge_indices()
        .map(|e| { 
            let (u, v) = g.edge_endpoints(e).unwrap();
            (perm[u.index()] , perm[v.index()])
        })
        .collect();

        let mut g = UnGraph::<usize, ()>::new_undirected();

        g.reserve_nodes(n);
        (0..n).for_each(|_| { g.add_node(1); });

        g.reserve_edges(edges.len());
        edges.into_iter().for_each(|(u, v)| { g.add_edge(NodeIndex::new(u), NodeIndex::new(v), ()); });

        return g
    }


    #[test]
    fn key_generation() {
        
        let g1 = gen_test_graph();
        let g2 = generate_permutated_graph(&g1);
        
        let key1 = GraphKey::new(&g1);
        let key2 = GraphKey::new(&g2);
        
        assert_eq!(key1, key2);
    }

    #[test]
    fn key_generation_large() {
        
        let g1 = generate_random_graph(2000, 0.05);
        let g2 = generate_permutated_graph(&g1);
        
        let key1 = GraphKey::new(&g1);
        let key2 = GraphKey::new(&g2);
        
        assert_eq!(key1, key2);
    }

    #[test]
    fn hashset_graphkeys() {
        
        let mut g = generate_random_graph(1000, 0.1);
        
        let g1 = generate_permutated_graph(&g);
        let g2 = generate_permutated_graph(&g);
        
        match g.find_edge(0.into(), 1.into()) {
            Some(_ix) => { g.remove_edge(_ix); }
            None => { g.add_edge(0.into(), 1.into(), ()); }
        }
        
        let g3 = generate_permutated_graph(&g);
        let g4 = generate_permutated_graph(&g);

        // generate Hashset
        let mut s = HashSet::new();

        s.insert(GraphKey::new(&g1));
        s.insert(GraphKey::new(&g2));
        s.insert(GraphKey::new(&g3));
        s.insert(GraphKey::new(&g4));

        assert_eq!(s.len(), 2);
    }


    #[test]
    fn is_isomorphic_test() {

        for _ in 0..100 {
            let g1 = generate_random_graph(500, 0.05);
            let g2 = generate_random_graph(500, 0.05);
            let g3 = generate_permutated_graph(&g1);

            let key1 = GraphKey::new(&g1);
            let key2 = GraphKey::new(&g2);
            let key3 = GraphKey::new(&g3);

            assert_eq!(is_isomorphic(&g1, &g2), key1 == key2);
            assert_eq!(key1, key3);
        }
    }


}
