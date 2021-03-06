// See LICENSE file for copyright and license details.

use common::types::{Size2, ZInt, PlayerId, MapPos};
use game_state::{GameState};
use map::{distance};
use pathfinder::{MapPath, Pathfinder};
use dir::{Dir};
use command::{Command, MoveMode};
use unit::{Unit};
use db::{Db};
use core::{CoreEvent, los};

pub struct Ai {
    id: PlayerId,
    state: GameState,
    pathfinder: Pathfinder,
}

impl Ai {
    pub fn new(id: &PlayerId, map_size: &Size2) -> Ai {
        Ai {
            id: id.clone(),
            state: GameState::new(map_size, id),
            pathfinder: Pathfinder::new(map_size),
        }
    }

    pub fn apply_event(&mut self, db: &Db, event: &CoreEvent) {
        self.state.apply_event(db, event);
    }

    // TODO: move fill_map here
    fn get_best_pos(&self) -> Option<MapPos> {
        let mut best_pos = None;
        let mut best_cost = None;
        for (_, enemy) in self.state.units() {
            if enemy.player_id == self.id {
                continue;
            }
            for i in 0 .. 6 {
                let dir = Dir::from_int(i);
                let destination = Dir::get_neighbour_pos(&enemy.pos, &dir);
                if !self.state.map().is_inboard(&destination) {
                    continue;
                }
                if self.state.is_tile_occupied(&destination) {
                    continue;
                }
                let path = match self.pathfinder.get_path(&destination) {
                    Some(path) => path,
                    None => continue,
                };
                let cost = path.total_cost().n;
                if let Some(ref mut best_cost) = best_cost {
                    if *best_cost > cost {
                        *best_cost = cost;
                        best_pos = Some(destination.clone());
                    }
                } else {
                    best_cost = Some(cost);
                    best_pos = Some(destination.clone());
                }
            }
        }
        best_pos
    }

    fn truncate_path(&self, path: MapPath, move_points: ZInt) -> MapPath {
        if path.total_cost().n <= move_points {
            return path;
        }
        let len = path.nodes().len();
        for i in 1 .. len {
            let cost = &path.nodes()[i].cost;
            if cost.n > move_points {
                let mut new_nodes = path.nodes().clone();
                new_nodes.truncate(i);
                return MapPath::new(new_nodes);
            }
        }
        assert!(false); // TODO
        return path;
    }

    fn is_close_to_enemies(&self, db: &Db, unit: &Unit) -> bool {
        for (_, target) in self.state.units() {
            if target.player_id == self.id {
                continue;
            }
            let max_distance = db.unit_max_attack_dist(unit);
            if distance(&unit.pos, &target.pos) <= max_distance {
                return true;
            }
        }
        false
    }

    pub fn try_get_move_command(&mut self, db: &Db) -> Option<Command> {
        for (_, unit) in self.state.units() {
            if unit.player_id != self.id {
                continue;
            }
            // println!("id: {}, ap: {}", unit.id.id, unit.attack_points);
            if unit.attack_points <= 0 {
                continue;
            }
            let unit_type = db.unit_type(&unit.type_id);
            for (_, target) in self.state.units() {
                if target.player_id == self.id {
                    continue;
                }
                let max_distance = db.unit_max_attack_dist(unit);
                if distance(&unit.pos, &target.pos) > max_distance {
                    continue;
                }
                if !los(self.state.map(), unit_type, &unit.pos, &target.pos) {
                    continue;
                }
                return Some(Command::AttackUnit {
                    attacker_id: unit.id.clone(),
                    defender_id: target.id.clone(),
                });
            }
        }
        None
    }

    pub fn try_get_attack_command(&mut self, db: &Db) -> Option<Command> {
        for (_, unit) in self.state.units() {
            if unit.player_id != self.id {
                continue;
            }
            if self.is_close_to_enemies(db, unit) {
                continue;
            }
            self.pathfinder.fill_map(db, &self.state, unit);
            let destination = match self.get_best_pos() {
                Some(destination) => destination,
                None => continue,
            };
            let path = match self.pathfinder.get_path(&destination) {
                Some(path) => path,
                None => continue,
            };
            if unit.move_points == 0 {
                continue;
            }
            let path = self.truncate_path(path, unit.move_points);
            return Some(Command::Move {
                unit_id: unit.id.clone(),
                path: path,
                mode: MoveMode::Fast,
            });
        }
        None
    }

    pub fn get_command(&mut self, db: &Db) -> Command {
        if let Some(cmd) = self.try_get_move_command(db) {
            cmd
        } else if let Some(cmd) = self.try_get_attack_command(db) {
            cmd
        } else {
            Command::EndTurn
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
