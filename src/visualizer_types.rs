// See LICENSE file for copyright and license details.

use gl::types::{GLfloat, GLuint};
use core_types::{ZInt};
use cgmath::{Vector3, Vector2};

#[deriving(Clone)]
pub struct Color3 {
    pub r: ZFloat,
    pub g: ZFloat,
    pub b: ZFloat,
}

#[deriving(Clone)]
pub struct Color4 {
    pub r: ZFloat,
    pub g: ZFloat,
    pub b: ZFloat,
    pub a: ZFloat,
}

pub type ZFloat = GLfloat;

#[deriving(Clone)]
pub struct VertexCoord{pub v: Vector3<ZFloat>}

/*
#[deriving(Clone)]
pub struct Normal{pub v: Vector3<ZFloat>}

#[deriving(Clone)]
pub struct TextureCoord{pub v: Vector2<ZFloat>}
*/

#[deriving(Clone)]
pub struct WorldPos{pub v: Vector3<ZFloat>}

#[deriving(Clone)]
pub struct ScreenPos{pub v: Vector2<ZInt>}

/*
pub struct Time{pub n: u64}
*/

#[deriving(Clone)]
pub struct MatId{pub id: GLuint}

#[deriving(Clone)]
pub struct ColorId{pub id: GLuint}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
