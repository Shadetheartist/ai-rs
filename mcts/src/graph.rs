use std::hash::Hash;
use petgraph::{Directed, Graph};
use petgraph::stable_graph::NodeIndex;
use rand::{Rng, RngCore, SeedableRng};
use crate::{Determinable, ismcts_mt};
use crate::ismcts::{ISMCTSParams};
use crate::mcts::{Mcts};

#[derive(Clone, Eq, PartialEq)]
pub struct GraphNode<S: Clone + Eq + PartialEq> {
    pub state: S,
}

#[derive(Clone, Eq, PartialEq)]
pub struct GraphEdge<A: Clone + Eq + PartialEq> {
    pub count: usize,
    pub action: A,
}

#[allow(dead_code)]
fn add_state_to_graph<S: Clone + Eq + PartialEq, A: Clone + Eq + PartialEq>(
    graph: &mut Graph<GraphNode<S>, GraphEdge<A>, Directed>,
    nodes: &mut [(NodeIndex, &S)],
    state: &S,
) -> NodeIndex {
    let node = GraphNode {
        state: state.clone(),
    };

    let node_index = {
        let existing = nodes.iter().find(|n| *n.1 == node.state);
        if let Some(existing) = existing {
            existing.0
        } else {
            graph.add_node(node.clone())
        }
    };

    node_index
}

#[allow(dead_code)]
fn add_action_to_graph<S: Clone + Eq + PartialEq, A: Clone + Eq + PartialEq>(
    graph: &mut Graph<GraphNode<S>, GraphEdge<A>, Directed>,
    action: A,
    prev_state_idx: NodeIndex,
    new_state_idx: NodeIndex,
) {
    let existing_edge = graph.find_edge(prev_state_idx, new_state_idx);
    if let Some(existing_edge) = existing_edge {
        let edge = graph.edge_weight(existing_edge).unwrap();
        graph.update_edge(prev_state_idx, new_state_idx, GraphEdge { action, count: edge.count + 1 });
    } else {
        graph.add_edge(prev_state_idx, new_state_idx, GraphEdge { action, count: 1 });
    }
}

pub trait Initializer<P, A: Send, S: Mcts<P, A>> {
    fn initialize<R: Rng + Sized>(&self, r: &mut R) -> S;
}

#[allow(dead_code)]
pub fn generate_graph<
    P: Eq + PartialEq + Hash + Send + Sync,
    A: Clone + Eq + PartialEq + Hash + Send + Sync,
    R: Rng + RngCore + Sized + Clone + Send + SeedableRng,
    G: Clone + Eq + PartialEq + Mcts<P, A> + Send + Determinable<P, A, G, R>,
    I: Initializer<P, A, G>
>
(initializer: &I, sim_params: ISMCTSParams) -> Graph<GraphNode<G>, GraphEdge<A>, Directed> {
    let mut graph: Graph<GraphNode<G>, GraphEdge<A>, Directed> = Graph::new();
    let mut nodes: Vec<(NodeIndex, &G)> = Vec::new();

    for sim_n in 0..sim_params.num_sims {
        let mut not_rng = R::seed_from_u64(sim_params.seed);
        let mut per_sim_rng = R::seed_from_u64(sim_params.seed + (sim_n as u64));

        let game = initializer.initialize(&mut not_rng);
        let players = game.players();

        #[allow(unused_variables)]
        let mut step = 0usize;

        add_state_to_graph(&mut graph, &mut nodes, &game);

        step += 1;

        loop {
            let current_player_idx = players.iter().enumerate().find(|(_, p)| **p == game.current_player()).unwrap().0;
            let sim_player = &sim_params.sim_players[current_player_idx];
            let ai_selected_action = ismcts_mt(&game, &per_sim_rng, sim_player.num_determinations, sim_player.num_simulations_per_action);

            let prev_node_idx = nodes.last().unwrap().0;

            let game = game.apply_action(ai_selected_action.clone(), &mut per_sim_rng).unwrap();

            let new_node_idx = add_state_to_graph(&mut graph, &mut nodes, &game);
            add_action_to_graph(&mut graph, ai_selected_action, prev_node_idx, new_node_idx);

            step += 1;

            if game.outcome().is_some() {
                break;
            }
        }
    }

    graph
}