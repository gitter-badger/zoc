// See LICENSE file for copyright and license details.

use std::rand::{thread_rng, Rng};
use cgmath::{Vector3, Vector, EuclideanVector, deg};
use visualizer::geom;
use core::types::{MapPos, UnitId};
use core::game_state::GameState;
use core::core;
use core::pathfinder::{MapPath};
use visualizer::mesh::{MeshId};
use visualizer::scene::{
    Scene,
    SceneNode,
    NodeId,
    MIN_MARKER_NODE_ID,
    SHELL_NODE_ID,
};
use visualizer::types::{ZFloat, WorldPos, Time};
use visualizer::unit_type_visual_info::{UnitTypeVisualInfo};

fn unit_id_to_node_id(unit_id: &UnitId) -> NodeId {
    NodeId{id: unit_id.id}
}

fn marker_id(unit_id: &UnitId) -> NodeId {
    NodeId{id: MIN_MARKER_NODE_ID.id + unit_id.id}
}

pub trait EventVisualizer {
    fn is_finished(&self) -> bool;
    fn draw(&mut self, scene: &mut Scene, dtime: &Time);
    fn end(&mut self, scene: &mut Scene, state: &GameState);
}

pub struct EventMoveVisualizer {
    unit_id: UnitId,
    path: Vec<WorldPos>,
    move_helper: MoveHelper,
    speed: ZFloat,
}

impl EventVisualizer for EventMoveVisualizer {
    fn is_finished(&self) -> bool {
        self.path.len() == 1
    }

    fn draw(&mut self, scene: &mut Scene, dtime: &Time) {
        let pos = self.move_helper.step(dtime);
        {
            let marker_node = scene.node(&marker_id(&self.unit_id));
            marker_node.pos.v = pos.v.add_v(&vec3_z(geom::HEX_EX_RADIUS / 2.0));
        }
        let node_id = unit_id_to_node_id(&self.unit_id);
        let node = scene.node(&node_id);
        node.pos = pos;
        if self.move_helper.is_finished() {
            let _ = self.path.remove(0);
            if self.path.len() > 1 {
                self.update_waypoint(node);
            }
            node.pos = self.current_waypoint().clone();
        }
    }

    fn end(&mut self, scene: &mut Scene, _: &GameState) {
        assert!(self.path.len() == 1);
        let node_id = unit_id_to_node_id(&self.unit_id);
        let node = scene.node(&node_id);
        node.pos = self.current_waypoint().clone();
    }
}

impl EventMoveVisualizer {
    pub fn new(
        scene: &mut Scene,
        _: &GameState,
        unit_id: UnitId,
        unit_type_visual_info: &UnitTypeVisualInfo,
        path: MapPath,
    ) -> Box<EventVisualizer + 'static> {
        let mut world_path = Vec::new();
        for &(_, ref map_pos) in path.nodes().iter() {
            let world_pos = geom::map_pos_to_world_pos(map_pos);
            world_path.push(world_pos);
        }
        let speed = unit_type_visual_info.move_speed;
        let node_id = unit_id_to_node_id(&unit_id);
        let node = scene.node(&node_id);
        node.rot = geom::get_rot_angle(
            &world_path[0], &world_path[1]);
        let move_helper = MoveHelper::new(
            world_path[0].clone(), world_path[1].clone(), speed);
        let mut vis = box EventMoveVisualizer {
            unit_id: unit_id.clone(),
            path: world_path,
            move_helper: move_helper,
            speed: speed,
        };
        vis.update_waypoint(node);
        vis as Box<EventVisualizer>
    }

    fn update_waypoint(&mut self, node: &mut SceneNode) {
        self.move_helper = MoveHelper::new(
            self.current_waypoint().clone(),
            self.next_waypoint().clone(),
            self.speed,
        );
        node.rot = geom::get_rot_angle(
            self.current_waypoint(),
            self.next_waypoint()
        );
    }

    fn current_waypoint(&self) -> &WorldPos {
        assert!(self.path.len() >= 1);
        &self.path[0]
    }

    fn next_waypoint(&self) -> &WorldPos {
        assert!(self.path.len() >= 2);
        &self.path[1]
    }
}

pub struct EventEndTurnVisualizer;

impl EventEndTurnVisualizer {
    pub fn new() -> Box<EventVisualizer+'static> {
        box EventEndTurnVisualizer as Box<EventVisualizer>
    }
}

impl EventVisualizer for EventEndTurnVisualizer {
    fn is_finished(&self) -> bool {
        true
    }

    fn draw(&mut self, _: &mut Scene, _: &Time) {}

    fn end(&mut self, _: &mut Scene, _: &GameState) {}
}

pub struct EventCreateUnitVisualizer {
    id: UnitId,
    move_helper: MoveHelper,
}

fn get_unit_scene_nodes(
    core: &core::Core,
    type_id: &core::UnitTypeId,
    mesh_id: &MeshId,
) -> Vec<SceneNode> {
    let count = core.object_types().get_unit_type(type_id).count;
    let mut vec = Vec::new();
    if count == 1 {
        vec![SceneNode {
            pos: WorldPos{v: Vector3{x: 0.0, y: 0.0, z: 0.0}},
            rot: deg(0.0),
            mesh_id: Some(mesh_id.clone()),
            children: vec![],
        }]
    } else {
        for i in range(0, count) {
            let pos = geom::index_to_circle_vertex(count, i).v.mul_s(0.3f32);
            vec.push(SceneNode {
                pos: WorldPos{v: pos},
                rot: deg(0.0),
                mesh_id: Some(mesh_id.clone()),
                children: vec![],
            });
        }
        vec
    }
}

impl EventCreateUnitVisualizer {
    pub fn new(
        core: &core::Core,
        scene: &mut Scene,
        _: &GameState,
        id: UnitId,
        type_id: &core::UnitTypeId,
        pos: &MapPos,
        mesh_id: &MeshId,
        marker_mesh_id: &MeshId,
    ) -> Box<EventVisualizer+'static> {
        let node_id = unit_id_to_node_id(&id);
        let world_pos = geom::map_pos_to_world_pos(pos);
        let to = world_pos;
        let from = WorldPos{v: to.v.sub_v(&vec3_z(geom::HEX_EX_RADIUS / 2.0))};
        let rot = deg(thread_rng().gen_range(0.0, 360.0));
        scene.nodes.insert(node_id, SceneNode {
            pos: from.clone(),
            rot: rot,
            mesh_id: None,
            children: get_unit_scene_nodes(core, type_id, mesh_id),
        });
        scene.nodes.insert(marker_id(&id), SceneNode {
            pos: WorldPos{v: to.v.add_v(&vec3_z(geom::HEX_EX_RADIUS / 2.0))},
            rot: deg(0.0),
            mesh_id: Some(marker_mesh_id.clone()),
            children: Vec::new(),
        });
        let move_helper = MoveHelper::new(from, to, 1.0);
        box EventCreateUnitVisualizer {
            id: id,
            move_helper: move_helper,
        } as Box<EventVisualizer>
    }
}

impl EventVisualizer for EventCreateUnitVisualizer {
    fn is_finished(&self) -> bool {
        self.move_helper.is_finished()
    }

    fn draw(&mut self, scene: &mut Scene, dtime: &Time) {
        let node_id = unit_id_to_node_id(&self.id);
        let node = scene.node(&node_id);
        node.pos = self.move_helper.step(dtime);
    }

    fn end(&mut self, _: &mut Scene, _: &GameState) {}
}

pub struct MoveHelper {
    to: WorldPos,
    current: WorldPos,
    dist: ZFloat,
    current_dist: ZFloat,
    dir: Vector3<ZFloat>,
}

impl MoveHelper {
    // TODO: speed: ZFloat -> Speed (add 'Speed' to src/visualizer/types.rs
    pub fn new(
        from: WorldPos,
        to: WorldPos,
        speed: ZFloat,
    ) -> MoveHelper {
        let dir = to.v.sub_v(&from.v).normalize();
        let dist = geom::dist(&from, &to);
        MoveHelper {
            to: to,
            current: from,
            dist: dist,
            current_dist: 0.0,
            dir: dir.mul_s(speed),
        }
    }

    pub fn is_finished(&self) -> bool {
        self.current_dist >= self.dist
    }

    pub fn step(&mut self, dtime: &Time) -> WorldPos {
        let dt = dtime.n as ZFloat / 1000000000.0;
        let step = self.dir.mul_s(dt);
        self.current_dist += step.length();
        self.current.v.add_self_v(&step);
        if self.is_finished() {
            self.current = self.to.clone();
        }
        self.current.clone()
    }
}

fn vec3_z(z: ZFloat) -> Vector3<ZFloat> {
    Vector3{x: 0.0, y: 0.0, z: z}
}

pub struct EventAttackUnitVisualizer {
    defender_id: UnitId,
    killed: bool,
    move_helper: MoveHelper,
    shell_move: MoveHelper,
}

impl EventAttackUnitVisualizer {
    pub fn new(
        scene: &mut Scene,
        _: &GameState,
        attacker_id: UnitId,
        defender_id: UnitId,
        killed: bool,
        shell_mesh_id: MeshId
    ) -> Box<EventVisualizer+'static> {
        let node_id = unit_id_to_node_id(&defender_id);
        let from = scene.nodes[node_id].pos.clone();
        let to = WorldPos{v: from.v.sub_v(&vec3_z(geom::HEX_EX_RADIUS / 2.0))};
        let move_helper = MoveHelper::new(from, to, 1.0);
        let shell_move = {
            let from = scene.nodes[unit_id_to_node_id(&attacker_id)].pos.clone();
            let to = scene.nodes[unit_id_to_node_id(&defender_id)].pos.clone();
            scene.nodes.insert(SHELL_NODE_ID, SceneNode {
                pos: from.clone(),
                rot: deg(0.0),
                mesh_id: Some(shell_mesh_id),
                children: Vec::new(),
            });
            MoveHelper::new(from, to.clone(), 10.0)
        };
        box EventAttackUnitVisualizer {
            defender_id: defender_id,
            killed: killed,
            move_helper: move_helper,
            shell_move: shell_move,
        } as Box<EventVisualizer>
    }
}

impl EventVisualizer for EventAttackUnitVisualizer {
    fn is_finished(&self) -> bool {
        if self.killed {
            self.move_helper.is_finished()
        } else {
            self.shell_move.is_finished()
        }
    }

    fn draw(&mut self, scene: &mut Scene, dtime: &Time) {
        scene.node(&SHELL_NODE_ID).pos = self.shell_move.step(dtime);
        if self.killed {
            if self.shell_move.is_finished() {
                let node_id = unit_id_to_node_id(&self.defender_id);
                scene.node(&node_id).pos = self.move_helper.step(dtime);
            }
        }
    }

    fn end(&mut self, scene: &mut Scene, _: &GameState) {
        if self.killed {
            let node_id = unit_id_to_node_id(&self.defender_id);
            scene.nodes.remove(&node_id);
            scene.nodes.remove(&marker_id(&self.defender_id));
        }
        scene.nodes.remove(&SHELL_NODE_ID);
    }
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab: