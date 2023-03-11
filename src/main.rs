
use petgraph::Graph;
use petgraph::graph::UnGraph;
use petgraph::Undirected;

use graphkey::coloring;

fn main() {

    let g = gen_test_graph();

    let tmp = coloring::Colouring::new(&g);

    tmp.print_cells();

}

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
