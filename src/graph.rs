/* Copyright (C) 2020 Yuval Deutscher

* This Source Code Form is subject to the terms of the Mozilla Public
* License, v. 2.0. If a copy of the MPL was not distributed with this
* file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use priority_queue::PriorityQueue;
use std::collections::HashMap;
use std::hash::Hash;

pub trait Weight {
    fn weight(&self) -> i64;
}

pub struct Node<N: Eq + Hash + Copy, E: Eq + Hash + Copy + Weight> {
    id: N,
    edges: HashMap<E, N>,
    best_cost: i64,
    best_prev_edge: Option<(N, E)>,
}

impl<N: Eq + Hash + Copy, E: Eq + Hash + Copy + Weight> Node<N, E> {
    pub fn new(id: N) -> Self {
        Node {
            id,
            edges: HashMap::new(),
            best_cost: i64::MAX,
            best_prev_edge: None,
        }
    }

    pub fn id(&self) -> &N {
        &self.id
    }

    pub fn edges(&self) -> impl Iterator<Item = (&E, &N)> {
        self.edges.iter()
    }

    pub fn connect(&mut self, edge: E, dest: N) {
        self.edges.insert(edge, dest);
    }
}

pub struct Graph<N: Eq + Hash + Copy, E: Eq + Hash + Copy + Weight> {
    nodes: HashMap<N, Node<N, E>>,
}

impl<N: Eq + Hash + Copy, E: Eq + Hash + Copy + Weight> Graph<N, E> {
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
        }
    }

    pub fn get(&self, id: &N) -> Option<&Node<N, E>> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: &N) -> Option<&mut Node<N, E>> {
        self.nodes.get_mut(id)
    }

    pub fn get_or_insert(&mut self, id: &N) -> &mut Node<N, E> {
        self.nodes.entry(*id).or_insert_with(|| Node::new(*id))
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Node<N, E>> {
        self.nodes.values()
    }

    fn dijkstra_init(&mut self, origin: &N) {
        for n in self.nodes.values_mut() {
            n.best_cost = i64::MAX;
            n.best_prev_edge = None;
        }
        self.nodes.get_mut(origin).unwrap().best_cost = 0;
    }

    fn dijkstra_core<T: Fn(&N) -> bool>(&mut self, origin: &N, predicate: T) -> Option<N> {
        let mut pq: PriorityQueue<N, i64> = PriorityQueue::new();
        pq.push(*origin, 0);
        while let Some((n, pr)) = pq.pop() {
            if predicate(&n) {
                return Some(n);
            }
            let node_ptr = self.nodes.get(&n).unwrap() as *const _;
            let node: &Node<N, E>;
            unsafe {
                node = std::mem::transmute(node_ptr);
            }
            debug_assert_eq!(node.best_cost, -pr);
            for (edge, n_dest) in node.edges() {
                let node_dest_ptr = self.nodes.get_mut(&n_dest).unwrap() as *mut _;
                let mut node_dest: &mut Node<N, E>;
                if node_ptr == node_dest_ptr {
                    assert!(&n == n_dest);
                    continue;
                } else {
                    unsafe {
                        node_dest = std::mem::transmute(node_dest_ptr);
                    }
                }
                let weight = edge.weight();
                assert!(weight >= 0);
                let cost = node.best_cost + weight;
                if cost < node_dest.best_cost {
                    node_dest.best_cost = cost;
                    node_dest.best_prev_edge = Some((n, *edge));
                    if pq.change_priority(n_dest, -cost).is_none() {
                        pq.push(*n_dest, -cost);
                    }
                }
            }
        }
        None
    }

    fn dijkstra_backtrace(&self, origin: &N, found: &N) -> Vec<(E, N)> {
        let mut result = Vec::new();
        let mut curr = *found;
        while &curr != origin {
            let (prev, edge) = self.nodes[&curr].best_prev_edge.unwrap();
            result.push((edge, curr));
            curr = prev;
        }
        result.reverse();
        result
    }

    pub fn find_shortest_path<T: Fn(&N) -> bool>(&mut self, origin: &N, predicate: T) -> Option<Vec<(E, N)>> {
        self.dijkstra_init(origin);
        let found = self.dijkstra_core(origin, predicate)?;
        Some(self.dijkstra_backtrace(origin, &found))
    }
}
