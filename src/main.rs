
use petgraph::Graph;
use petgraph::graph::{UnGraph, NodeIndex};
use petgraph::Undirected;
use petgraph::algo::is_isomorphic;

use graphkey::GraphKey;
use std::time::Instant;

fn main() {

    use std::env;
    env::set_var("RUST_BACKTRACE", "1");

    // let g1 = gen_test_graph();
    let g1 = generate_random_graph(4000, 0.1);
    let g2 = generate_permutated_graph(&g1);

    let start = Instant::now();
    let key1 = GraphKey::new(&g1);
    let key2 = GraphKey::new(&g2);
    let are_isomorphic_graphkey = key1 == key2;
    let duration_graphkey = start.elapsed();

    let start = Instant::now();
    let are_isomorphic_petgraph = is_isomorphic(&g1, &g2);
    let duration_petgraph = start.elapsed();

    println!("Isomorphis check with petgraph : {} ({:?})", are_isomorphic_petgraph, duration_petgraph);
    println!("Isomorphis check with graphkey : {} ({:?})", are_isomorphic_graphkey, duration_graphkey);
}

#[allow(dead_code)]
fn gen_test_graph() -> Graph::<usize, (), Undirected> {
    
    let edges : Vec<(u32, u32)> = vec![
        (0, 3), (0, 5), (0, 8),
        (1, 4), (1, 6), (1, 8),
        (2, 5), (2, 7),
        (3, 6), (3, 9),
        (4, 7), (4, 9),
        (5, 8), (7, 9)
    ];

    return UnGraph::from_edges(edges);
}

fn generate_random_graph(n : usize, p : f64) -> Graph::<usize, (), Undirected> {

    use rand::Rng;

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



















use rand::thread_rng;
use rand::seq::SliceRandom;

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

    let mut _g = UnGraph::<usize, ()>::new_undirected();

    _g.reserve_nodes(n);
    (0..n).for_each(|_| { _g.add_node(1); });

    _g.reserve_edges(edges.len());
    edges.into_iter().for_each(|(u, v)| { _g.add_edge(NodeIndex::new(u), NodeIndex::new(v), ()); });

    return _g
}