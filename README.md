# graphkey-rust
An implementation of the Traces algorithm in Rust to enable fast graph isomorphish check and the Hash trait on Graph objects. For a complete description of the algorithm, see [this publication](https://arxiv.org/pdf/0804.4881.pdf).

## Usage
Given a graph (from the awesome petgraph library), this algorithm computes the key of the graph. This key is isomorphic-invariant, hence it can be hashed & used for isomorphism test.

```rust
use petgraph::{Graph, graph::Undirected};
use graphkey::GraphKey;

fn main() {
    
    let edges : Vec<(u32, u32)> = vec![
        (0, 3), (0, 5), (0, 8), (1, 4), (1, 6), (1, 8),
        (2, 5), (2, 7), (3, 6), (3, 9),
        (4, 7), (4, 9), (5, 8), (7, 9)
    ];

    let g = Graph::<usize, (), Undirected>::from_edges(edges);
    let key = GraphKey::new(&g);
}
```

## Isomorphism check
The resulting key is stable by permutation, and thus can be used for an isomorphism check.

```rust
fn main() {
    
    let edges : Vec<(u32, u32)> = vec![
        (0, 3), (0, 5), (0, 8), (1, 4), (1, 6), (1, 8),
        (2, 5), (2, 7), (3, 6), (3, 9),
        (4, 7), (4, 9), (5, 8), (7, 9)
    ];

    let g1 = Graph::<usize, (), Undirected>::from_edges(edges);
    let g2 = generate_permutated_graph(&g1);
    
    let key1 = GraphKey::new(&g1);
    let key2 = GraphKey::new(&g2);
    
    assert!(key1 == key2);
}
```

## Using GraphKey as key in HashSet
As a wrapper around a Vec<usize>, GraphKey implements the Hash trait.

```rust
use std::collections::HashSet;

fn main() {

    let edges1 : Vec<(u32, u32)> = vec![
        (0, 3), (0, 5), (0, 8), (1, 4), (1, 6), (1, 8),
        (2, 5), (2, 7), (3, 6), (3, 9),
        (4, 7), (4, 9), (5, 8), (7, 9)
    ];

    let edges2 : Vec<(u32, u32)> = vec![
        (0, 3), (0, 5), (0, 9), (1, 4), (1, 6), (1, 8),
        (2, 5), (2, 7), (3, 6), (3, 9),
        (4, 7), (4, 9), (5, 8), (7, 9)
    ];
    
    let g1 = Graph::<usize, (), Undirected>::from_edges(edges1);
    let g2 = generate_permutated_graph(&g1);
    
    let g3 = Graph::<usize, (), Undirected>::from_edges(edges2);
    let g4 = generate_permutated_graph(&g3);

    let mut s = HashSet::new();

    s.insert(GraphKey::new(&g1));
    s.insert(GraphKey::new(&g2));
    s.insert(GraphKey::new(&g3));
    s.insert(GraphKey::new(&g4));
    
    assert_eq!(s.len(), 2);
}
```

## Performence of the isomorphism check against petgraph::algo::is_isomorphic

For large graphs (n > 1_000), the key comparison allows to perform an isomorphism check faster than with the algorithm provided currently in petgraph. In particular, it can handle large graphs graphs ( > 10_000 nodes) which is not possible with petgraph::algo::is_isomorphic.

```rust
fn main() {

    use petgraph::algo::is_isomorphic;

    let g1 = generate_random_graph(5000, 0.1);
    let g2 = generate_permutated_graph(&g1);

    let start = Instant::now();
    let key1 = GraphKey::new(&g1);
    let key2 = GraphKey::new(&g2);
    let are_isomorphic_graphkey = key1 == key2;
    let duration_graphkey = start.elapsed();

    let start = Instant::now();
    let are_isomorphic_petgraph = is_isomorphic(&g1, &g2);
    let duration_petgraph = start.elapsed();

    println!("Isomorphism check with petgraph : {} ({:?})", are_isomorphic_petgraph, duration_petgraph);
    println!("Isomorphism check with graphkey : {} ({:?})", are_isomorphic_graphkey, duration_graphkey);
}
```

```bash
Isomorphism check with petgraph : true (50.6870042s)
Isomorphism check with graphkey : true (1.5694743s)
```

Note that the complexity of the isomorphism check is highly dependant of the graph structure.

## Bonus : the generate_permutated_graph and generate_random_graph functions.

```rust
fn generate_permutated_graph(g : &Graph::<usize, (), Undirected>) -> Graph::<usize, (), Undirected> {
    
    use rand::thread_rng;
    use rand::seq::SliceRandom;
    
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
```


