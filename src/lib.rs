pub mod coloring;

//
//
//

use petgraph::{graph::Graph, Undirected};

fn lib_001() {
    use petgraph;

    // petgraph::algo::is_isomorphic(g0, g1)
}

//
// GraphKey object
//

#[derive(Hash, PartialEq, Eq)]
pub struct GraphKey(Vec<usize>);

impl GraphKey {
    pub fn new(g : Graph::<usize, (), Undirected>) -> GraphKey {
        return GraphKey(vec![0]);
    }
}




//
// are_isomorphic
//

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
