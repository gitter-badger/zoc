// See LICENSE file for copyright and license details.

#![feature(old_path, old_io)] // TODO

extern crate cgmath;

#[cfg(target_os = "android")]
extern crate android_glue;

pub mod fs;
pub mod misc;
pub mod types;

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
