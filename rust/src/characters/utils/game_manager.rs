use crate::{CONNECTION_STATE, GLOBAL_CONNECTION};
use godot::classes::{INode, Label, Node};
use godot::prelude::*;
use std::sync::atomic::Ordering;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct GameManager {
    position_update_timer: f32,
    position_update_interval: f32,

    score_label: Option<Gd<Label>>,
    coins: Option<Gd<Node>>,
    coins_number: i32,

    #[base]
    base: Base<Node>,
}

#[godot_api]
impl INode for GameManager {
    fn init(base: Base<Node>) -> Self {
        Self {
            position_update_timer: 0.0,
            position_update_interval: 2.0, // 60 FPS position updates
            score_label: None,
            coins: None,
            coins_number: 0,
            base,
        }
    }

    fn ready(&mut self) {
        self.score_label = self.base().try_get_node_as::<Label>("ScoreLabel");
        self.coins = self.base().get_node_or_null("%Coins");

        if let Some(coins_node) = &self.coins {
            self.coins_number = coins_node.get_child_count();
        }

        self.update_score_display();
    }

    fn process(&mut self, delta: f64) {
        self.position_update_timer += delta as f32;
        if self.position_update_timer >= self.position_update_interval
            && CONNECTION_STATE.load(Ordering::SeqCst) == 2
        {
            self.refresh_display();
            self.position_update_timer = 0.0;
        }
    }
}

#[godot_api]
impl GameManager {
    #[func]
    pub fn get_total_coins_collected(&self) -> i32 {
        let connection = GLOBAL_CONNECTION.lock().unwrap();
        let scores = connection.get_all_scores();

        scores
            .iter()
            .map(|score| score.coins_collected as i32)
            .sum()
    }

    #[func]
    pub fn refresh_display(&mut self) {
        self.update_score_display();
    }

    fn update_score_display(&mut self) {
        let connection = GLOBAL_CONNECTION.lock().unwrap();
        let scores = connection.get_all_scores();
        let total_collected = scores.iter().map(|s| s.coins_collected as i32).sum::<i32>();

        if let Some(score_label) = &mut self.score_label {
            let text = if scores.is_empty() {
                format!("No coins collected yet! (0/{})", self.coins_number)
            } else if scores.len() == 1 {
                let score = &scores[0];
                format!(
                    "{}: {}/{} coins!",
                    score.player_name, score.coins_collected, self.coins_number
                )
            } else {
                let mut player_list = Vec::new();

                for score in &scores {
                    player_list.push(format!("{}: {}", score.player_name, score.coins_collected));
                }
                player_list.sort(); // Sort alphabetically

                format!(
                    "Total: {}/{} coins! [{}]",
                    total_collected,
                    self.coins_number,
                    player_list.join(", ")
                )
            };

            score_label.set_text(&text);
        } else {
            godot_warn!("ScoreLabel not available to update");
        }
    }
}
