use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
struct Node {
    val: String,
    children: Vec<Node>,
}

impl Node {
    fn new(val: String, first_child: String) -> Self {
        Self {
            val,
            children: vec![Self {
                val: first_child,
                children: vec![],
            }],
        }
    }

    /// Return a path from this node to the requested node
    /// if one exists. For example,
    /// ```
    ///                    D -> E -> F
    ///                   /
    /// COM -> A -> B -> C-> G -> H
    ///                       \
    ///                        I -> J -> K
    /// ```
    /// Calling (where `node` contains `"COM"`) `node.path("J")`
    /// should return:
    ///  `Some(vec!["COM", "A", "B", "C", "G", "I", "J"])`
    fn path(&self, to: &str) -> Option<Vec<&str>> {
        if self.val == to {
            Some(vec![&self.val])
        } else {
            for child in &self.children {
                match child.path(to) {
                    Some(mut path) => {
                        let mut ret: Vec<&str> = vec![&self.val];
                        ret.append(&mut path);
                        return Some(ret);
                    }
                    None => (),
                }
            }
            None
        }
    }

    /// Calculate the sum of all direct and indirect orbits
    /// of all bodies under this node.
    fn depth(&self) -> usize {
        fn inner_depth(node: &Node, depth: usize) -> usize {
            depth
                + node
                    .children
                    .iter()
                    .map(|c| inner_depth(c, 1 + depth))
                    .sum::<usize>()
        }

        self.children.iter().map(|c| inner_depth(c, 1)).sum()
    }

    /// If a node exists with the supplied key, return a mutable
    /// reference to it.
    fn find_node_mut(&mut self, key: &str) -> Option<&mut Node> {
        if self.val == key {
            Some(self)
        } else {
            self.children.iter_mut().find_map(|n| n.find_node_mut(key))
        }
    }

    /// Given another node, find a spot within this node
    /// and merge them together. Returns true if a match
    /// was found and the merge was completed.
    fn merge(&mut self, other: &mut Node) -> bool {
        if let Some(node) = self.find_node_mut(&other.val) {
            node.children.append(&mut other.children);
            true
        } else {
            false
        }
    }
}

fn parse() -> Vec<Node> {
    let file = File::open("day6/input.txt").expect("Unable to open input file");
    let mut reader = BufReader::new(file);
    let mut buffer = String::new();
    let mut read = reader
        .read_line(&mut buffer)
        .expect("failed to read line from file");
    let mut nodes = vec![];
    while read > 0 {
        let keys: Vec<_> = buffer
            .split(")")
            .filter_map(|s| {
                let s = s.trim();
                if s.len() == 0 {
                    None
                } else {
                    Some(s)
                }
            })
            .collect();
        if keys.len() != 2 {
            break;
        }

        let bigger_mass = keys[0].to_owned();
        let smaller_mass = keys[1].to_owned();
        nodes.push(Node::new(bigger_mass.clone(), smaller_mass.clone()));

        buffer.clear();
        read = reader
            .read_line(&mut buffer)
            .expect("failed to read line from file");
    }

    nodes
}

// Creates an lookup table of orbitted body names to indices in
// the original node list
fn bigger_mass_index(nodes: &Vec<Node>) -> HashMap<String, Vec<usize>> {
    let mut index = HashMap::new();
    for (i, node) in nodes.iter().enumerate() {
        index.entry(node.val.clone()).or_insert(vec![]).push(i);
    }

    index
}

// Create a histogram of the number of times a body
// is find in the node list.
fn histogram(nodes: &Vec<Node>) -> HashMap<String, usize> {
    let mut histogram = HashMap::new();
    for node in nodes {
        *histogram.entry(node.val.clone()).or_insert(0) += 1;
        for child in &node.children {
            *histogram.entry(child.val.clone()).or_insert(0) += 1;
        }
    }

    histogram
}

fn process(mut nodes: Vec<Node>) -> (usize, isize) {
    // 1. Figure out which node is the root node
    //   a. This can be done by finding the number of
    //      times a body is listed in the original
    //      input nodes, then filtering out to only
    //      the ones on the left side, and which have
    //      a count of one. If we didn't account for the
    //      ones on the left, we'd also get all the lea..
    //      "moon" nodes (XD) and not be able to tell
    //      the difference between those and the root
    //      node
    // 2. Add the root node to a process queue
    // 3. For every item that's added to the queue:
    //   a. Pop that node off the front of the queue
    //   b. Find all of its children in the index (an O(1) op)
    //   c. Merge each child of that "moon" node (should be
    //      small big O since it's already at the "moon" node)
    //   d. Add its newly acquired child nodes to the back of
    //      the queue
    // 4. This should iterate to a fixed point, and then we
    //    should be left with just the root node and its
    //    fully assembled orbit :)

    let index = bigger_mass_index(&nodes);
    let mut root_node = {
        let mut histogram = histogram(&nodes);
        // Only retain histogram for bodies that are orbitted. We don't
        // care about moon nodes for this count ;).
        histogram.retain(|k, v| index.contains_key(k) && *v == 1);
        assert_eq!(
            histogram.len(),
            1,
            "There should have been only 1 root node!"
        );
        let root_str = histogram.iter().next().unwrap().0;
        let root_indices = &index[root_str];
        assert_eq!(root_indices.len(), 1, "I messed up real bad...");
        nodes.remove(root_indices[0])
    };

    // recompute so indices in loop below are correct
    let index = bigger_mass_index(&nodes);
    let mut queue = VecDeque::<&mut Node>::new();
    queue.push_back(&mut root_node);
    while let Some(node) = queue.pop_front() {
        let indices_to_merge: Vec<_> = node
            .children
            .iter()
            .map(|child| index.get(&child.val).map(|v| v.iter()))
            // flatten to remove the None's
            .flatten()
            // flatten the iter of vec iters of usize into iter of usize
            .flatten()
            .map(|u| *u)
            .collect();

        for i in indices_to_merge {
            let merged = node.merge(&mut nodes[i]);
            assert!(merged);
        }

        for child in &mut node.children {
            queue.push_back(child);
        }
    }

    // Sanity check to make sure we got them all
    nodes.retain(|n| n.children.len() > 0);
    assert_eq!(nodes.len(), 0);

    // Now that we have the fully assembled orbits, we can compute the depth
    // (see the returned values at the end of this fn), and the path between
    // two moons.

    // Find paths from the root node to the wanted path
    let path_to_you = root_node.path("YOU").expect("Should have a path to YOU");
    let path_to_san = root_node.path("SAN").expect("Should have a path to SAN");
    let mut i = 0;

    // Now we want to remove the parts of the paths that are the same, e.g.
    //                         E -> F -> G -> YOU
    //                        /
    // COM -> A -> B -> C -> D -> H -> I
    //                             \
    //                              J -> K -> SAN
    // We want to remove COM through D, so that we end up looking at E and H
    while i < path_to_you.len() && i < path_to_san.len() {
        if path_to_you[i] != path_to_san[i] {
            break;
        }

        i += 1;
    }

    // As long as we actually found something similar, just add the remaining
    // lengths and account for the fact that YOU and SAN are on the planets
    // (the -2 part).
    let jumps = if i < path_to_you.len() && i < path_to_san.len() {
        (path_to_you[i..].len() + path_to_san[i..].len() - 2) as isize
    } else {
        -1
    };

    (root_node.depth(), jumps)
}

fn main() {
    println!("{:?}", process(parse()));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn merge_returns_true_when_merge_occurred() {
        let mut n1 = Node::new("A".to_owned(), "B".to_owned());
        let mut n2 = Node::new("B".to_owned(), "C".to_owned());

        assert!(n1.merge(&mut n2));
        assert_eq!(&n1.val, "A");
        assert_eq!(n1.children.len(), 1);
        assert_eq!(&n1.children[0].val, "B");
        assert_eq!(n1.children[0].children.len(), 1);
        assert_eq!(&n1.children[0].children[0].val, "C");
        assert_eq!(n1.children[0].children[0].children.len(), 0);
        assert_eq!(&n2.val, "B");
        assert_eq!(n2.children.len(), 0);
    }

    #[test]
    fn merge_returns_true_when_nested_merge_occurred() {
        let mut n1 = Node::new("A".to_owned(), "B".to_owned());
        let mut n2 = Node::new("B".to_owned(), "C".to_owned());
        let mut n3 = Node::new("C".to_owned(), "D".to_owned());

        assert!(n1.merge(&mut n2));
        assert!(n1.merge(&mut n3));
        assert_eq!(&n1.val, "A");
        assert_eq!(n1.children.len(), 1);
        assert_eq!(&n1.children[0].val, "B");
        assert_eq!(n1.children[0].children.len(), 1);
        assert_eq!(&n1.children[0].children[0].val, "C");
        assert_eq!(n1.children[0].children[0].children.len(), 1);
        assert_eq!(&n1.children[0].children[0].children[0].val, "D");
        assert_eq!(n1.children[0].children[0].children[0].children.len(), 0);
        assert_eq!(&n3.val, "C");
        assert_eq!(n3.children.len(), 0);
    }

    #[test]
    fn merge_returns_true_when_not_first_merge_occurred() {
        let mut n1 = Node::new("A".to_owned(), "B".to_owned());
        let mut n2 = Node::new("A".to_owned(), "C".to_owned());

        assert!(n1.merge(&mut n2));
        assert_eq!(&n1.val, "A");
        assert_eq!(n1.children.len(), 2);
        assert_eq!(&n1.children[0].val, "B");
        assert_eq!(n1.children[0].children.len(), 0);
        assert_eq!(&n1.children[1].val, "C");
        assert_eq!(n1.children[1].children.len(), 0);
        assert_eq!(&n2.val, "A");
        assert_eq!(n2.children.len(), 0);
    }

    #[test]
    fn merge_returns_true_when_nested_not_first_merge_occurred() {
        let mut n1 = Node::new("A".to_owned(), "B".to_owned());
        let mut n2 = Node::new("B".to_owned(), "C".to_owned());
        let mut n3 = Node::new("B".to_owned(), "D".to_owned());

        assert!(n1.merge(&mut n2));
        assert!(n1.merge(&mut n3));
        assert_eq!(&n1.val, "A");
        assert_eq!(n1.children.len(), 1);
        assert_eq!(&n1.children[0].val, "B");
        assert_eq!(n1.children[0].children.len(), 2);
        assert_eq!(&n1.children[0].children[0].val, "C");
        assert_eq!(n1.children[0].children[0].children.len(), 0);
        assert_eq!(&n1.children[0].children[1].val, "D");
        assert_eq!(n1.children[0].children[1].children.len(), 0);
        assert_eq!(&n3.val, "B");
        assert_eq!(n3.children.len(), 0);
    }

    #[test]
    fn merge_returns_false_when_no_merge_occurred() {
        let mut n1 = Node::new("A".to_owned(), "B".to_owned());
        let mut n2 = Node::new("C".to_owned(), "D".to_owned());
        assert!(!n1.merge(&mut n2));
        assert_eq!(&n1.val, "A");
        assert_eq!(n1.children.len(), 1);
        assert_eq!(n1.children[0].val, "B");
        assert_eq!(n1.children[0].children.len(), 0);
        assert_eq!(&n2.val, "C");
        assert_eq!(n2.children.len(), 1);
        assert_eq!(n2.children[0].val, "D");
        assert_eq!(n2.children[0].children.len(), 0);
    }

    #[test]
    fn depth_test() {
        let mut n1 = Node::new("COM".to_owned(), "B".to_owned());
        let mut n2 = Node::new("B".to_owned(), "C".to_owned());
        let mut n3 = Node::new("C".to_owned(), "D".to_owned());
        let mut n4 = Node::new("D".to_owned(), "E".to_owned());
        let mut n5 = Node::new("E".to_owned(), "F".to_owned());
        let mut n6 = Node::new("B".to_owned(), "G".to_owned());
        let mut n7 = Node::new("G".to_owned(), "H".to_owned());
        let mut n8 = Node::new("D".to_owned(), "I".to_owned());
        let mut n9 = Node::new("E".to_owned(), "J".to_owned());
        let mut n10 = Node::new("J".to_owned(), "K".to_owned());
        let mut n11 = Node::new("K".to_owned(), "L".to_owned());

        assert!(n1.merge(&mut n2));
        assert!(n1.merge(&mut n3));
        assert!(n1.merge(&mut n4));
        assert!(n1.merge(&mut n5));
        assert!(n1.merge(&mut n6));
        assert!(n1.merge(&mut n7));
        assert!(n1.merge(&mut n8));
        assert!(n1.merge(&mut n9));
        assert!(n1.merge(&mut n10));
        assert!(n1.merge(&mut n11));

        println!("{:#?}", n1);

        assert_eq!(n1.depth(), 42);
    }

    // TODO(pfaria) Should probably add tests for process...
}
