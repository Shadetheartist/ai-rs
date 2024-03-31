use petgraph::{Directed, Graph};
use petgraph::stable_graph::NodeIndex;
use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;
use crate::ismcts::{ismcts, ISMCTSParams};
use crate::mcts::MCTS;


pub trait Graphable {

}

#[derive(Clone, Eq, PartialEq)]
pub struct GraphNode<S: Clone + Eq + PartialEq> {
    pub state: S,
}

#[derive(Clone, Eq, PartialEq)]
pub struct GraphEdge<A: Clone + Eq + PartialEq> {
    pub count: usize,
    pub action: A,
}


fn add_state_to_graph<S: Clone + Eq + PartialEq, A: Clone + Eq + PartialEq>(
    graph: &mut Graph<GraphNode<S>, GraphEdge<A>, Directed>,
    nodes: &mut Vec<(NodeIndex, &S)>,
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

fn add_action_to_graph<S: Clone + Eq + PartialEq, A: Clone + Eq + PartialEq>(
    graph: &mut Graph<GraphNode<S>, GraphEdge<A>, Directed>,
    action: A,
    prev_state_idx: NodeIndex,
    new_state_idx: NodeIndex,
) {
    let existing_edge = graph.find_edge(prev_state_idx, new_state_idx);
    if let Some(existing_edge) = existing_edge {
        let edge = graph.edge_weight(existing_edge).unwrap();
        graph.update_edge(prev_state_idx, new_state_idx, GraphEdge { action: action, count: edge.count + 1 });
    } else {
        graph.add_edge(prev_state_idx, new_state_idx, GraphEdge { action: action, count: 1 });
    }
}

pub trait Initializer<'p, P, A: Send, S: MCTS<'p, P, A>> {
    fn initialize<R: Rng + Sized>(&self, r: &mut R) -> S;
}

pub fn generate_graph<'p,
    S: Clone + Eq + PartialEq + MCTS<'p, P, A>,
    A: Clone + Eq + PartialEq + Send,
    P,
    I: Initializer<'p, P, A, S>
>(initializer: &I, sim_params: ISMCTSParams) -> Graph<GraphNode<S>, GraphEdge<A>, Directed> {
    let mut graph: Graph<GraphNode<S>, GraphEdge<A>, Directed> = Graph::new();
    let mut nodes: Vec<(NodeIndex, &S)> = Vec::new();

    for sim_n in 0..sim_params.num_sims {
        let mut not_rng = Pcg64::seed_from_u64(sim_params.seed);
        let mut per_sim_rng = Pcg64::seed_from_u64(sim_params.seed + (sim_n as u64));

        let mut game_state = initializer.initialize(&mut not_rng);
        let mut step = 0usize;

        add_state_to_graph(&mut graph, &mut nodes, &game_state);

        step += 1;

        loop {
            let sim_player = &sim_params.sim_players[game_state.current_player()];
            let ai_selected_action = ismcts(&game_state, &mut per_sim_rng, sim_player.num_determinations, sim_player.num_simulations_per_action);

            let prev_node_idx = nodes.last().unwrap().0;

            game_state = game_state.apply_action(ai_selected_action.clone(), &mut per_sim_rng).unwrap();

            let new_node_idx = add_state_to_graph(&mut graph, &game_state);
            add_action_to_graph(&mut graph, ai_selected_action.clone(), prev_node_idx, new_node_idx);

            step += 1;

            if let Some(_winner) = game_state.winner() {
                break;
            }
        }
    }

    graph
}