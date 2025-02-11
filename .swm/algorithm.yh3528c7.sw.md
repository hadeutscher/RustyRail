---
title: Algorithm
---
At its core, HaRail uses [Dijkstra's algorithm](https://en.wikipedia.org/wiki/Dijkstra%27s_algorithm) on a [directed acyclic graph](https://en.wikipedia.org/wiki/Directed_acyclic_graph) to find the shortest routes. However, exactly how the graph is constructed is nontrivial.

<SwmSnippet path="/lib/src/graph.rs" line="88">

---

Core implementation of Dijkstra's algorithm. Note that the destination node is not given definitively but by predicate, because we actually search for the shortest path to one of a set of possible nodes.

```renderscript
    fn dijkstra_core<T: Fn(&N) -> bool>(
        &self,
        origin: &Node<N, E>,
        predicate: T,
        distances: &mut HashMap<N, NodeDistance<N, E>>,
    ) -> Option<N> {
        let mut pq: PriorityQueue<N, i64> = PriorityQueue::new();
        pq.push(origin.id, 0);
        while let Some((n, pr)) = pq.pop() {
            if predicate(&n) {
                return Some(n);
            }
            let node = self.nodes.get(&n).unwrap();
            let node_best_cost = -pr;
            debug_assert_eq!(distances[&n].best_cost, node_best_cost);
            for (edge, n_dest) in node.edges() {
                let weight = edge.weight();
                assert!(weight >= 0);
                let cost = node_best_cost + weight;
                let node_dest_distance = distances.get_mut(n_dest).unwrap();
                if cost < node_dest_distance.best_cost {
                    node_dest_distance.best_cost = cost;
                    node_dest_distance.best_prev_edge = Some((n, *edge));
                    if pq.change_priority(n_dest, -cost).is_none() {
                        pq.push(*n_dest, -cost);
                    }
                }
            }
        }
        None
    }
```

---

</SwmSnippet>

&nbsp;

## Node definition

At first thought it seems as if mapping train routes to a graph would mean that the graph's nodes should describe stations, and the edges should describe trains. However, this is inadequate for the purpose of building a schedule. A "shortest path" on such a graph would be a sequence of trains that provide the least cumulative time *while on the train* but completely disregard how much waiting is needed between trains. It may even provide a sequence of trains that requires time travel. To incorporate the idea that trains leave a station at a point in time, we must think of a node as a (Station, Time) tuple:

<SwmSnippet path="/lib/src/lib.rs" line="80">

---

The Singularity object constitutes graph nodes in HaRail. The importance of the <SwmToken path="/lib/src/lib.rs" pos="84:1:1" line-data="    train: Option&lt;&amp;&#39;a Train&gt;,">`train`</SwmToken> field will be addressed later.

```renderscript
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
struct Singularity<'a> {
    station: &'a Station,
    time: NaiveDateTime,
    train: Option<&'a Train>,
}
```

---

</SwmSnippet>

Edges (trains) then connect stations *at certain times*. This makes sense: it might be possible to ride a train from A at 16:00 to B at 17:00, but this train might not necessarily be available at other times.

This definition, however, now only allows us to concatenate trains that leave and arrive at *exactly* the same time. To adjust for the fact that we can wait for a train and then ride it, we need to add a few more edges. A neat way to do this is to make the edge type a sum type of two possible actions - riding a train or waiting in station.

<SwmSnippet path="/lib/src/lib.rs" line="87">

---

The Action object that constitutes graph edges. The other action types will also be addressed shortly.

```renderscript
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
enum Action<'a> {
    Wait(Duration),
    TrainWaits(&'a Train, Stop<'a>),
    Ride(&'a Train, Stop<'a>, Stop<'a>),
    Board(&'a Train),
    Unboard,
}
```

---

</SwmSnippet>

We then need to create "waiting in station" edges connecting nodes that represent the same station at different points in time. To do this while adding the minimal number of "wait" edges to the graph, we inspect the sorted set of all times where the station has train arrivals or departures and connect each pair of successive times with a "wait" edge. This allows us to wait at the station from any train arrival to any train departure that is at a later time, by jumping over "wait" edges until we reach the node that's connected to the departing train.

<SwmSnippet path="/lib/src/lib.rs" line="206">

---

Connecting the different time points of every station with wait edges.

```renderscript
        // Connect each station's singularities with wait edges
        for (_, station_set) in stations_general {
            let mut station_vec: Vec<Singularity> = station_set.into_iter().collect();
            station_vec.sort_unstable_by_key(|s| s.time);
            let mut prev = None;
            for curr in station_vec {
                if let Some(prev) = prev {
                    result
                        .get_mut(&prev)
                        .unwrap()
                        .connect(Action::Wait(curr.time - prev.time), curr);
                }
                prev = Some(curr);
            }
        }
```

---

</SwmSnippet>

There is one additional Singularity that we must consider here, even though it doesn't have any arrivals or departures: the time at which the user arrives at their start station. This also happens to be our origin node for Dijkstra's algorithm.&nbsp;

<SwmSnippet path="lib/src/lib.rs" line="404">

---

Adding an origin node of the starting station at the starting time.

```
    let origin = Singularity {
        station: start_station,
        time: start_time,
        train: None,
    };
    g.ensure(origin);
```

---

</SwmSnippet>

<SwmSnippet path="/lib/src/lib.rs" line="225">

---

The <SwmToken path="/lib/src/lib.rs" pos="409:3:3" line-data="    g.ensure(origin);">`ensure`</SwmToken> function inserts the node into the graph if it doesn't already exist and hooks it into the chain of wait edges for the station.

```renderscript
    fn ensure(&mut self, s: Singularity<'a>) {
        if self.get(&s).is_none() {
            self.get_or_insert(&s);
            if let Some(next) = self
                .nodes()
                .map(|n| n.id())
                .filter(|n| n.train == s.train && n.station == s.station && n.time > s.time)
                .min_by_key(|n| n.time)
                .copied()
            {
                self.get_mut(&s)
                    .unwrap()
                    .connect(Action::Wait(next.time - s.time), next);
            }
            if let Some(prev) = self
                .nodes()
                .map(|n| n.id())
                .filter(|n| n.train == s.train && n.station == s.station && n.time < s.time)
                .max_by_key(|n| n.time)
                .copied()
            {
                self.get_mut(&prev)
                    .unwrap()
                    .connect(Action::Wait(s.time - prev.time), s);
            }
        }
    }
```

---

</SwmSnippet>

<SwmMeta version="3.0.0" repo-id="Z2l0aHViJTNBJTNBUnVzdHlSYWlsJTNBJTNBaGFkZXV0c2NoZXI=" repo-name="RustyRail"><sup>Powered by [Swimm](https://app.swimm.io/)</sup></SwmMeta>
