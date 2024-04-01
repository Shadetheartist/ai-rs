#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;
use coup_rs::{Action, Coup};
use eframe::egui;
use egui::emath;
use egui_graphs::{DefaultEdgeShape, DefaultNodeShape, Graph, GraphView, SettingsInteraction, SettingsNavigation, SettingsStyle};
use egui_plot::{Line, PlotPoints};
use petgraph::{Directed};
use petgraph::prelude::{EdgeIndex, NodeIndex};
use serde::Serialize;
use mcts::{Determinable, GraphEdge, GraphNode, Initializer, ISMCTSParams, ISMCTSPlayerParams};

fn main() -> Result<(), eframe::Error> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        "MCTS Explorer",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::<MCTSExplorer<usize, Action, Coup>>::default()
        }),
    )
}

#[derive(Clone)]
struct MCTSPlayerParams {
    enabled: bool,
    num_determinations: usize,
    num_simulations_per_action: usize,
}

struct MCTSParams {
    seed: u64,
    num_sims: usize,
    sim_players: Vec<MCTSPlayerParams>,
}

struct MTCTSweepDatum {
    sweep_index: usize,
    sim_params: MCTSParams,
    winners: Vec<(usize, i32)>,
}

struct MCTSExplorer<P: Clone, A: Clone + Eq, G: mcts::Mcts<P, A> + Eq> {
    selected_node_idx: Option<NodeIndex>,
    seed: u64,
    num_sims: usize,
    params: Vec<MCTSPlayerParams>,
    graph: Option<Graph<GraphNode<G>, GraphEdge<A>, Directed>>,
    show_graph: bool,
    sweep_data: Option<Vec<MTCTSweepDatum>>,
    phantom_p: PhantomData<P>
}

impl<
    P: Eq + PartialEq + Hash + Send + Sync + Clone,
    A: Eq + PartialEq + Hash + Send + Sync + Clone + Debug,
    G: mcts::Mcts<P, A> + Determinable<P, A, G> + Initializer<P, A, G> + Eq +  Send + Sync ,
> MCTSExplorer<P, A, G> {
    fn sim_params(&self) -> MCTSParams {
        MCTSParams {
            seed: self.seed,
            num_sims: self.num_sims,
            sim_players: self.params
                .clone()
                .into_iter()
                .filter(|p| p.enabled)
                .collect(),
        }
    }
    /*
    fn graph_winners(&self) -> Option<Vec<(usize, i32)>> {
        if let Some(graph) = &self.graph {
            let default_map = (0..self.sim_params().sim_players.len()).fold(HashMap::default(), |mut acc, n| {
                acc.insert(n, 0);
                acc
            });

            let winner_map = graph.nodes_iter().map(|n| n.0).filter_map(|n_idx| {
                let node = graph.node(n_idx).unwrap();
                node.payload().state.winner()
            }).fold(default_map, |mut acc, n| {
                *acc.entry(n).or_insert(0) += 1;
                acc
            });

            let mut winner_vec: Vec<(usize, i32)> = winner_map.iter().map(|(k, v)| { (*k, *v) }).collect();
            winner_vec.sort_by(|a, b| {
                a.0.partial_cmp(&b.0).unwrap()
            });

            Some(winner_vec)
        } else {
            None
        }
    }*/

    fn coup_graph(&self) -> Graph<GraphNode<G>, GraphEdge<A>, Directed>
        where
            P: Eq + PartialEq + Hash + Send + Sync + Clone,
            A: Eq + PartialEq + Hash + Send + Sync + Debug,
            G: Determinable<P, A, G> + Initializer<P, A, G> + Eq + PartialEq + Send + Sync,
    {
        let game_graph = mcts::generate_graph::<P, A, rand_pcg::Lcg128Xsl64, G, G>(ISMCTSParams{
            seed: self.seed,
            num_sims: self.num_sims,
            max_cores: 0,
            sim_players: vec![
                ISMCTSPlayerParams { num_determinations: 4, num_simulations_per_action: 10 },
                ISMCTSPlayerParams { num_determinations: 4, num_simulations_per_action: 10 },
                ISMCTSPlayerParams { num_determinations: 4, num_simulations_per_action: 10 },
                ISMCTSPlayerParams { num_determinations: 4, num_simulations_per_action: 10 },
            ],
        });

        let mut graph = Graph::from(&game_graph);

        let node_indexes: Vec<NodeIndex> = graph.nodes_iter().map(|n| n.0).collect();

        node_indexes.iter().for_each(|idx| {
            let (sim, step) = {
                let node = graph.node_mut(*idx).unwrap();
                (node.payload().sim, node.payload().step)
            };

            let node = graph.node_mut(*idx).unwrap();
            node.set_label("".to_string());
            node.set_location(emath::Pos2 { x: (sim * 200) as f32, y: (step * 50 + sim * 10) as f32 });
        });

        let edge_indexes: Vec<EdgeIndex> = graph.edges_iter().map(|n| n.0).collect();
        edge_indexes.iter().for_each(|idx| {
            let edge_action = graph.g.edge_weight(*idx).unwrap().clone();
            let edge = graph.edge_mut(*idx).unwrap();
            edge.set_label(format!("   {:?}, n={:?}", edge_action.payload().action, edge_action.payload().count));
        });

        graph
    }

    fn sweep(&mut self) -> Vec<MTCTSweepDatum> {
        let mut data: Vec<MTCTSweepDatum> = Vec::new();

        let initial_seed = self.seed;

        for p in &mut self.params {
            p.enabled = false;
            p.num_simulations_per_action = 4;
            p.num_determinations = 50;
        }

        self.params[0].enabled = true;
        self.params[1].enabled = true;
        self.params[2].enabled = true;

        for idx in 0..10 {
            self.seed += 1;
            self.params[0].num_simulations_per_action = 1 + idx * 50;

            let sim_params = self.sim_params();
            self.graph = Some(self.coup_graph());
            data.push(MTCTSweepDatum {
                sweep_index: idx,
                sim_params,
                winners: vec![]//self.graph_winners().unwrap(),
            });
        }

        self.seed = initial_seed;

        data
    }
}


impl<P: Clone, A: Clone + Eq, G: mcts::Mcts<P, A> + Eq> Default for MCTSExplorer<P, A, G> {
    fn default() -> Self {
        Self {
            show_graph: true,
            selected_node_idx: None,
            seed: 0,
            num_sims: 10,
            params: vec![
                MCTSPlayerParams {
                    enabled: true,
                    num_determinations: 1,
                    num_simulations_per_action: 1,
                },
                MCTSPlayerParams {
                    enabled: true,
                    num_determinations: 1,
                    num_simulations_per_action: 1,
                },
                MCTSPlayerParams {
                    enabled: true,
                    num_determinations: 1,
                    num_simulations_per_action: 1,
                },
                MCTSPlayerParams {
                    enabled: false,
                    num_determinations: 1,
                    num_simulations_per_action: 1,
                },
                MCTSPlayerParams {
                    enabled: false,
                    num_determinations: 1,
                    num_simulations_per_action: 1,
                },
                MCTSPlayerParams {
                    enabled: false,
                    num_determinations: 1,
                    num_simulations_per_action: 1,
                },
            ],
            graph: None,
            sweep_data: None,
            phantom_p: Default::default(),
        }
    }
}

impl<P: Clone, A: Clone + Eq, G: mcts::Mcts<P, A> + Eq> MCTSExplorer<P, A, G> {
    fn read_data(&mut self) {
        if let Some(graph) = &self.graph {
            if !graph.selected_nodes().is_empty() {
                let idx = graph.selected_nodes().first().unwrap();
                self.selected_node_idx = Some(*idx);
            }
        }
    }
}

impl<
    P: Eq + PartialEq + Hash + Send + Sync + Clone,
    A: Eq + PartialEq + Hash + Send + Sync + Clone + Debug,
    G: mcts::Mcts<P, A> + Determinable<P, A, G> + Initializer<P, A, G> + Eq +  Send + Sync + Debug + Serialize,
> eframe::App for MCTSExplorer<P, A, G> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.read_data();

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            ui.vertical(|vert| {
                vert.add(egui::Checkbox::new(&mut self.show_graph, "Show Graph"));
            });

            ui.vertical(|vert| {
                vert.label("Seed");
                vert.add(egui::Slider::new(&mut self.seed, 1..=100));
            });

            ui.vertical(|vert| {
                vert.label("Num Sims");
                vert.add(egui::Slider::new(&mut self.num_sims, 1..=100));
            });


            ui.vertical(|vert| {
                for (idx, param) in &mut self.params.iter_mut().enumerate() {
                    vert.add(egui::Checkbox::new(&mut param.enabled, format!("Player {idx}")));
                    vert.add(egui::Slider::new(&mut param.num_determinations, 1..=240));
                    vert.add(egui::Slider::new(&mut param.num_simulations_per_action, 1..=1000));
                }

                if vert.button("Simulate").clicked() {
                    self.graph = Some(self.coup_graph());
                }
            });

            ui.vertical(|vert| {
                if vert.button("Sweep").clicked() {
                    self.sweep_data = Some(self.sweep());
                }
            });
        });

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            if let Some(idx) = self.selected_node_idx {
                if let Some(graph) = &self.graph {
                    if let Some(node) = graph.node(idx) {
                        ui.label(format!("{:?}", node.payload().state));

                        let actions = node.payload().state.actions().iter().fold("".to_string(), |acc, a| format!("{}\n{:?}", acc, a));
                        ui.label(actions);

                        let state_json = serde_json::to_string(&node.payload().state).unwrap();
                        ui.label(&state_json);
                    }
                }
            }
        });

        egui::TopBottomPanel::bottom("bottom_panel").exact_height(500.0).show(ctx, |ui| {
           /*
            if let Some(winners) = &self.graph_winners() {
                egui::Grid::new("table").show(ui, |ui| {
                    for (player_idx, _) in winners {
                        ui.label(format!("Player {player_idx}"));
                    }
                    ui.end_row();

                    for (_, win_count) in winners {
                        ui.label(format!("{win_count}"));
                    }
                    ui.end_row();
                });
            }*/

            if let Some(sweep_data) = &self.sweep_data {
                egui_plot::Plot::new("plot").show(ui, |ui| {
                    let mut player_line_points: Vec<Vec<[f64; 2]>> = Vec::new();

                    for _ in 0..sweep_data[0].sim_params.sim_players.len() {
                        player_line_points.push(vec![]);
                    }

                    for datum in sweep_data {
                        for (player_idx, points) in player_line_points.iter_mut().enumerate().take(datum.sim_params.sim_players.len()) {
                            let wins = datum.winners.iter().find(|w| w.0 == player_idx).unwrap().1;
                            points.push([datum.sweep_index as f64, wins as f64]);
                        }
                    }

                    for (player_idx, points) in player_line_points.iter().enumerate() {
                        ui.line(Line::new(PlotPoints::new(points.clone())).name(format!("player {player_idx}")))
                    }
                });
            }
        });

        if self.show_graph {
            egui::CentralPanel::default().show(ctx, |ui| {
                let interaction_settings = &SettingsInteraction::new()
                    .with_dragging_enabled(true)
                    .with_node_clicking_enabled(true)
                    .with_node_selection_enabled(true)
                    .with_node_selection_multi_enabled(false)
                    .with_edge_clicking_enabled(false)
                    .with_edge_selection_enabled(false)
                    .with_edge_selection_multi_enabled(false);
                let style_settings = &SettingsStyle::new().with_labels_always(true);
                let nav_settings = &SettingsNavigation::new().with_fit_to_screen_enabled(false).with_zoom_and_pan_enabled(true);

                ui.vertical(|vert| {
                    if let Some(graph) = &mut self.graph {
                        let mut view = GraphView::<_, _, _, _, DefaultNodeShape, DefaultEdgeShape>::new(graph)
                            .with_styles(style_settings)
                            .with_navigations(nav_settings)
                            .with_interactions(interaction_settings);

                        vert.add(
                            &mut view
                        );
                    }
                });
            });
        }
    }
}