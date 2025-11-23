#![allow(dead_code)]

use std::mem::size_of;

use wgpu::{BufferAddress, VertexBufferLayout, VertexStepMode};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Position(pub f32, pub f32);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Color(pub f32, pub f32, pub f32, pub f32);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Size(pub f32, pub f32);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Globals {
    pub resolution: Size,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct WidgetInstance {
    pub position: Position,
    pub size: Size,
    pub color: Color,
    pub radius: f32,
}

impl Position {
    const ATTRIBS: [wgpu::VertexAttribute; 1] = wgpu::vertex_attr_array![0 => Float32x2];

    pub const fn descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

impl From<wgpu::Color> for Color {
    fn from(value: wgpu::Color) -> Self {
        Self(
            value.r as f32,
            value.g as f32,
            value.b as f32,
            value.a as f32,
        )
    }
}

impl WidgetInstance {
    const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        1 => Float32x2, // position: Location 1
        2 => Float32x2, // size: Location 2
        3 => Float32x4, // color: Location 3
        4 => Float32,   // radius: Location 4
    ];

    pub fn new(x: f32, y: f32, width: f32, height: f32, color: Color, radius: f32) -> Self {
        Self {
            position: Position(x, y),
            size: Size(width, height),
            color,
            radius,
        }
    }

    pub const fn descriptor() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}
