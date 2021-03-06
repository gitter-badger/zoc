// See LICENSE file for copyright and license details.

use common::types::{PlayerId, MapPos, Size2, ZInt};
use core::{CoreEvent};
use internal_state::{InternalState};
use map::{Map, Terrain, distance};
use fov::{fov};
use db::{Db};
use unit::{Unit, UnitType, UnitClass};

#[derive(Clone, PartialEq, PartialOrd)]
pub enum TileVisibility {
    No,
    // Bad,
    Normal,
    Excellent,
}

pub fn fov_unit(
    db: &Db,
    terrain: &Map<Terrain>,
    fow: &mut Map<TileVisibility>,
    unit: &Unit,
) {
    fov_unit_in_pos(db, terrain, fow, unit, &unit.pos);
}

pub fn fov_unit_in_pos(
    db: &Db,
    terrain: &Map<Terrain>,
    fow: &mut Map<TileVisibility>,
    unit: &Unit,
    origin: &MapPos,
) {
    let unit_type = db.unit_type(&unit.type_id);
    let range = &unit_type.los_range;
    fov(
        terrain,
        origin,
        *range,
        &mut |pos| {
            let distance = distance(origin, pos);
            let vis = calc_visibility(terrain.tile(pos), unit_type, &distance);
            if vis > *fow.tile_mut(pos) {
                *fow.tile_mut(pos) = vis;
            }
        },
    );
}

fn calc_visibility(terrain: &Terrain, unit_type: &UnitType, distance: &ZInt)
    -> TileVisibility
{
    if *distance <= unit_type.cover_los_range {
        TileVisibility::Excellent
    } else if *distance <= unit_type.los_range {
        match terrain {
            &Terrain::Trees => TileVisibility::Normal,
            &Terrain::Plain => TileVisibility::Excellent,
        }
    } else {
        TileVisibility::No
    }
}

/// Fog of War
pub struct Fow {
    map: Map<TileVisibility>,
    player_id: PlayerId,
}

impl Fow {
    pub fn new(map_size: &Size2, player_id: &PlayerId) -> Fow {
        Fow {
            map: Map::new(map_size, TileVisibility::No),
            player_id: player_id.clone(),
        }
    }

    pub fn is_tile_visible(&self, pos: &MapPos) -> bool {
        match *self.map.tile(pos) {
            TileVisibility::Excellent => true,
            TileVisibility::Normal => true,
            TileVisibility::No => false,
        }
    }

    pub fn is_visible(&self, unit_type: &UnitType, pos: &MapPos) -> bool {
        match *self.map.tile(pos) {
            TileVisibility::Excellent => true,
            TileVisibility::Normal => match unit_type.class {
                UnitClass::Infantry => false,
                UnitClass::Vehicle => true,
            },
            TileVisibility::No => false,
        }
    }

    fn clear(&mut self) {
        for pos in self.map.get_iter() {
            *self.map.tile_mut(&pos) = TileVisibility::No;
        }
    }

    fn reset(&mut self, db: &Db, state: &InternalState) {
        self.clear();
        for (_, unit) in state.units() {
            if unit.player_id == self.player_id {
                fov_unit(db, state.map(), &mut self.map, &unit);
            }
        }
    }

    pub fn apply_event(
        &mut self,
        db: &Db,
        state: &InternalState,
        event: &CoreEvent,
    ) {
        match event {
            &CoreEvent::Move{ref unit_id, ref path, ..} => {
                let unit = state.unit(unit_id);
                if unit.player_id == self.player_id {
                    for path_node in path.nodes() {
                        let p = &path_node.pos;
                        fov_unit_in_pos(
                            db, state.map(), &mut self.map, unit, p);
                    }
                }
            },
            &CoreEvent::EndTurn{ref new_id, ..} => {
                if self.player_id == *new_id {
                    self.reset(db, state);
                }
            },
            &CoreEvent::CreateUnit{ref unit_id, ref player_id, ..} => {
                let unit = state.unit(unit_id);
                if self.player_id == *player_id {
                    fov_unit(db, state.map(), &mut self.map, unit);
                }
            },
            &CoreEvent::AttackUnit{..} => {},
            &CoreEvent::ShowUnit{..} => {},
            &CoreEvent::HideUnit{..} => {},
        }
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
