// See LICENSE file for copyright and license details.

use std::collections::{HashMap};
use cgmath::{Vector2};
use common::types::{PlayerId, UnitId, MapPos, Size2};
use core::{CoreEvent, FireMode};
use unit::{Unit, UnitTypeId};
use db::{Db};
use map::{Map, Terrain};
use command::{MoveMode};

pub enum InfoLevel {
    Full,
    Partial,
}

pub struct InternalState {
    units: HashMap<UnitId, Unit>,
    map: Map<Terrain>,
}

impl<'a> InternalState {
    pub fn new(map_size: &Size2) -> InternalState {
        let mut map = Map::new(map_size, Terrain::Plain);
        // TODO: read from scenario.json?
        *map.tile_mut(&MapPos{v: Vector2{x: 4, y: 3}}) = Terrain::Trees;
        *map.tile_mut(&MapPos{v: Vector2{x: 4, y: 4}}) = Terrain::Trees;
        *map.tile_mut(&MapPos{v: Vector2{x: 4, y: 5}}) = Terrain::Trees;
        *map.tile_mut(&MapPos{v: Vector2{x: 5, y: 5}}) = Terrain::Trees;
        *map.tile_mut(&MapPos{v: Vector2{x: 6, y: 4}}) = Terrain::Trees;
        InternalState {
            units: HashMap::new(),
            map: map,
        }
    }

    pub fn units(&self) -> &HashMap<UnitId, Unit> {
        &self.units
    }

    pub fn unit(&'a self, id: &UnitId) -> &'a Unit {
        &self.units[id]
    }

    pub fn map(&'a self) -> &Map<Terrain> {
        &self.map
    }

    pub fn units_at(&'a self, pos: &MapPos) -> Vec<&'a Unit> {
        let mut units = Vec::new();
        for (_, unit) in &self.units {
            if unit.pos == *pos {
                units.push(unit);
            }
        }
        units
    }

    pub fn is_tile_occupied(&self, pos: &MapPos) -> bool {
        // TODO: optimize
        self.units_at(pos).len() > 0
    }

    /// Converts active ap (attack points) to reactive
    fn convert_ap(&mut self, player_id: &PlayerId) {
        for (_, unit) in self.units.iter_mut() {
            if unit.player_id == *player_id {
                if let Some(ref mut reactive_attack_points)
                    = unit.reactive_attack_points
                {
                    *reactive_attack_points += unit.attack_points;
                }
                unit.attack_points = 0;
            }
        }
    }

    fn refresh_units(&mut self, db: &Db, player_id: &PlayerId) {
        for (_, unit) in self.units.iter_mut() {
            if unit.player_id == *player_id {
                let unit_type = db.unit_type(&unit.type_id);
                unit.move_points = unit_type.move_points;
                unit.attack_points = unit_type.attack_points;
                if let Some(ref mut reactive_attack_points) = unit.reactive_attack_points {
                    *reactive_attack_points = unit_type.reactive_attack_points;
                }
                unit.morale += 10;
            }
        }
    }

    fn add_unit(
        &mut self,
        db: &Db,
        unit_id: &UnitId,
        pos: &MapPos,
        type_id: &UnitTypeId,
        player_id: &PlayerId,
        info_level: InfoLevel,
    ) {
        assert!(self.units.get(unit_id).is_none());
        let unit_type = db.unit_type(type_id);
        self.units.insert(unit_id.clone(), Unit {
            id: unit_id.clone(),
            pos: pos.clone(),
            player_id: player_id.clone(),
            type_id: type_id.clone(),
            move_points: unit_type.move_points,
            attack_points: unit_type.attack_points,
            reactive_attack_points: if let InfoLevel::Full = info_level {
                Some(unit_type.reactive_attack_points)
            } else {
                None
            },
            count: unit_type.count,
            morale: 100,
        });
    }

    pub fn apply_event(&mut self, db: &Db, event: &CoreEvent) {
        match event {
            &CoreEvent::Move{ref unit_id, ref path, ref mode} => {
                let pos = path.destination().clone();
                let unit = self.units.get_mut(unit_id)
                    .expect("Bad move unit id");
                unit.pos = pos;
                assert!(unit.move_points > 0);
                if let &MoveMode::Fast = mode {
                    unit.move_points -= path.total_cost().n;
                } else {
                    unit.move_points -= path.total_cost().n * 2;
                }
                assert!(unit.move_points >= 0);
            },
            &CoreEvent::EndTurn{ref new_id, ref old_id} => {
                self.refresh_units(db, new_id);
                self.convert_ap(old_id);
            },
            &CoreEvent::CreateUnit {
                ref unit_id,
                ref pos,
                ref type_id,
                ref player_id,
            } => {
                self.add_unit(db, unit_id, pos, type_id, player_id, InfoLevel::Full);
            },
            &CoreEvent::AttackUnit {
                ref attacker_id,
                ref defender_id,
                ref mode,
                ref killed,
                ref suppression,
                ref remove_move_points,
            } => {
                {
                    let unit = self.units.get_mut(defender_id)
                        .expect("Can`t find defender");
                    unit.count -= *killed;
                    unit.morale -= *suppression;
                    if *remove_move_points {
                        unit.move_points = 0;
                    }
                }
                let count = self.units[defender_id].count.clone();
                if count <= 0 {
                    assert!(self.units.get(defender_id).is_some());
                    self.units.remove(defender_id);
                }
                let attacker_id = match attacker_id.clone() {
                    Some(attacker_id) => attacker_id,
                    None => return,
                };
                if let Some(unit) = self.units.get_mut(&attacker_id) {
                    match mode {
                        &FireMode::Active => {
                            assert!(unit.attack_points >= 1);
                            unit.attack_points -= 1;
                        },
                        &FireMode::Reactive => {
                            if let Some(ref mut reactive_attack_points)
                                = unit.reactive_attack_points
                            {
                                assert!(*reactive_attack_points >= 1);
                                *reactive_attack_points -= 1;
                            }
                        },
                    }
                }
            },
            &CoreEvent::ShowUnit{
                ref unit_id,
                ref pos,
                ref type_id,
                ref player_id,
            } => {
                self.add_unit(db, unit_id, pos, type_id, player_id, InfoLevel::Partial);
            },
            &CoreEvent::HideUnit{ref unit_id} => {
                assert!(self.units.get(unit_id).is_some());
                self.units.remove(unit_id);
            },
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
