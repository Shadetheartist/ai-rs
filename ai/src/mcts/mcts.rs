use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use rand::{Rng, RngCore};
use crate::mcts::Outcome;
use crate::random_rollout;

pub trait Mcts<P, A>: Clone {
    type Error: Debug;

    fn actions(&self) -> Vec<A>;
    fn apply_action<R: Rng + Sized>(&self, action: A, rng: &mut R) -> Result<Self, Self::Error> where Self: Sized;
    fn outcome(&self) -> Option<Outcome<P>>;

    fn current_player(&self) -> P;
    fn players(&self) -> Vec<P>;
}


pub struct VecTree<P, A, G: Mcts<P, A>> {
    current_player: P,
    nodes: Vec<VecTreeNode<P, A, G>>,
    phantom_p: PhantomData<P>,
    phantom_a: PhantomData<A>,
}

impl<P: Eq + PartialEq + Hash + Clone, A: Clone, G: Mcts<P, A>> VecTree<P, A, G> {
    pub fn from_state(state: G) -> Self {
        let mut tree = VecTree {
            current_player: state.current_player(),
            nodes: vec![],
            phantom_p: Default::default(),
            phantom_a: Default::default(),
        };

        tree.add_node(state, None);

        tree
    }

    pub fn search_n<R: Rng>(&mut self, rng: &mut R, iterations: usize) {
        for _ in 0..iterations {
            self.search(rng);
        }
    }

    pub fn best_action(&self) -> Option<A> {
        if let Some(root) = self.nodes.first() {
            let best_child = root.children.iter().enumerate().fold(None, |acc: Option<(usize, f64)>, (action_idx, child_node_idx): (usize, &usize)| {
                let node = &self.nodes[*child_node_idx];
                let player_value = if let Some(value) = node.value.get(&self.current_player) {
                    *value
                } else {
                    0f64
                };
                let avg_value = player_value / node.num_visits;

                if let Some(acc) = acc {
                    if avg_value > acc.1 {
                        return Some((action_idx, avg_value));
                    }
                }

                // default
                Some((action_idx, avg_value))
            });

            if let Some(best_child) = best_child {
                let actions = root.state.actions();
                return Some(actions[best_child.0].clone());
            }
        }

        None
    }

    pub fn search<R: Rng>(&mut self, rng: &mut R) {
        let mut current_node_idx = 0;

        // track visited nodes for back propagation
        let mut visited_nodes = Vec::new();
        visited_nodes.push(current_node_idx);

        // recursively select an optimal node to expand
        while self.nodes[current_node_idx].is_leaf() == false {
            current_node_idx = self.select(current_node_idx, rng);
            visited_nodes.push(current_node_idx);
        }

        let result = if let Some(outcome) = self.nodes[current_node_idx].state.outcome() {
            outcome
        } else {
            self.expand(current_node_idx, rng);

            let new_node_idx = self.select(current_node_idx, rng);
            visited_nodes.push(new_node_idx);

            random_rollout(&self.nodes[new_node_idx].state, rng)
        };

        match result {
            Outcome::Winner(winner_player) => {
                for visited_node_idx in visited_nodes {
                    self.nodes[visited_node_idx].num_visits += 1.0;
                    *self.nodes[visited_node_idx].value.entry(winner_player.clone()).or_insert(0f64) += 1.0;
                }
            }
            Outcome::Winners(winner_players) => {
                for visited_node_idx in visited_nodes {
                    self.nodes[visited_node_idx].num_visits += 1.0;
                    for winner_player in &winner_players {
                        *self.nodes[visited_node_idx].value.entry(winner_player.clone()).or_insert(0f64) += 1.0;
                    }
                }
            }
            Outcome::Escape(_) => {}
        }


    }

    fn expand<R: Rng>(&mut self, node_idx: usize, rng: &mut R) {
        let actions = {
            let node = &self.nodes[node_idx];
            node.state.actions()
        };

        if actions.len() == 0 {
            panic!("no actions to expand into")
        }

        for action in actions {
            let node = &self.nodes[node_idx];
            let state = node.state.apply_action(action.clone(), rng).unwrap();
            self.add_node(state, Some(node_idx));
        }
    }

    fn select<R: Rng>(&self, node_idx: usize, rng: &mut R) -> usize {
        let node = &self.nodes[node_idx];

        let constant_of_exploration = 2f64.sqrt();

        let selected = node.children.iter().fold((None, f64::MIN), |acc, child_idx| {
            let ucb = self.ucbt_value(*child_idx, constant_of_exploration, rng);
            if ucb > acc.1 {
                (Some(*child_idx), ucb)
            } else {
                acc
            }
        });

        if let Some(selected) = selected.0 {
            selected
        } else {
            panic!("could not select")
        }
    }

    fn ucbt_value<R: Rng>(&self, node_idx: usize, constant_of_exploration: f64, rng: &mut R) -> f64 {
        let node = &self.nodes[node_idx];

        if node.num_visits == 0.0 {
            return f64::MAX;
        }

        let player_value = if let Some(value) = node.value.get(&self.current_player) {
            *value
        } else {
            0f64
        };

        // first component of UCB1 formula corresponds to exploitation
        // as it is high for moves with a high averate win ratio
        // this is the average reward, or win ratio, of the node
        let exploitation_component = player_value / node.num_visits;

        // the second component corresponds to exploration
        let parent_vists = self.parent_visits(node);
        let exploration_component = constant_of_exploration * ((parent_vists + 1.0).ln() / node.num_visits).sqrt();
        let noise = rng.next_u32() as f64 * 1e-6;

        exploitation_component + exploration_component + noise
    }

    fn parent_visits(&self, node: &VecTreeNode<P, A, G>) -> f64 {
        if let Some(parent_idx) = node.parent_idx {
            let parent = &self.nodes[parent_idx];
            parent.num_visits
        } else {
            0f64
        }
    }

    fn add_node(&mut self, state: G, parent_idx: Option<usize>) -> usize {
        let next_idx = self.nodes.len();

        let node = if let Some(parent_idx) = parent_idx {
            let node = VecTreeNode::from_state_with_parent_idx(state, parent_idx);
            self.nodes[parent_idx].children.push(next_idx);
            node
        } else {
            VecTreeNode::from_state(state)
        };

        self.nodes.push(node);

        next_idx
    }
}

pub struct VecTreeNode<P, A, G: Mcts<P, A>> {
    num_visits: f64,
    value: HashMap<P, f64>,
    state: G,
    parent_idx: Option<usize>,
    children: Vec<usize>,
    phantom_a: PhantomData<A>,
}


impl<P, A: Clone, G: Mcts<P, A>> VecTreeNode<P, A, G> {
    fn from_state(state: G) -> Self {
        VecTreeNode {
            num_visits: 0.0,
            value: HashMap::new(),
            state: state,
            parent_idx: None,
            children: Vec::new(),
            phantom_a: Default::default(),
        }
    }

    fn from_state_with_parent_idx(state: G, parent: usize) -> Self {
        VecTreeNode {
            num_visits: 0.0,
            value: HashMap::new(),
            state: state,
            parent_idx: Some(parent),
            children: Vec::new(),
            phantom_a: Default::default(),
        }
    }

    fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }
}

pub fn mcts<
    R: Rng + RngCore + Sized,
    P: Eq + PartialEq + Hash + Clone + Send,
    A: Eq + PartialEq + Hash + Clone + Debug,
    G: Mcts<P, A>
>(game: &G, rng: &mut R, num_simulations: usize) -> A {
    let mut tree = VecTree::from_state(game.clone());

    // damn waste of a populated tree after this function closes
    tree.search_n(rng, num_simulations);

    tree.best_action().expect("an action")
}
