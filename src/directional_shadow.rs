use bevy::{
    prelude::*,
    render::{
        render_resource::{ShaderType, UniformBuffer},
        renderer::{RenderDevice, RenderQueue},
        Extract, RenderApp, RenderStartup,
    },
};

use crate::directional_light::DirectionalLight2d;

#[derive(Clone, ShaderType)]
pub struct ExtractedDirectionalLight2d {
    pub color: LinearRgba,
    pub direction: Vec2,
    pub tile_size: f32,
    pub shadow_length: f32,
    pub strength: f32,
}

impl Default for ExtractedDirectionalLight2d {
    fn default() -> Self {
        Self {
            color: LinearRgba::WHITE,
            direction: Vec2::ZERO,
            tile_size: 64.0,
            shadow_length: 0.0,
            strength: 0.0,
        }
    }
}

impl From<&DirectionalLight2d> for ExtractedDirectionalLight2d {
    fn from(l: &DirectionalLight2d) -> Self {
        let shadow_length = if l.strength > 0.0 {
            l.tile_size / l.strength.max(0.001)
        } else {
            0.0
        };
        Self {
            color: l.color.to_linear(),
            direction: if l.direction.length_squared() > f32::EPSILON {
                l.direction.normalize()
            } else {
                Vec2::ZERO
            },
            tile_size: l.tile_size.max(1.0),
            shadow_length,
            strength: l.strength.clamp(0.0, 1.0),
        }
    }
}

#[derive(Resource)]
pub struct DirectionalLight2dUniform {
    pub buffer: UniformBuffer<ExtractedDirectionalLight2d>,
}

impl Default for DirectionalLight2dUniform {
    fn default() -> Self {
        let mut buffer = UniformBuffer::default();
        buffer.set(ExtractedDirectionalLight2d::default());
        Self { buffer }
    }
}

pub fn extract_directional_light(
    mut uniform: ResMut<DirectionalLight2dUniform>,
    query: Extract<Query<&DirectionalLight2d>>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    let extracted = query
        .iter()
        .next()
        .map(ExtractedDirectionalLight2d::from)
        .unwrap_or_default();

    uniform.buffer.set(extracted);
    uniform.buffer.write_buffer(&render_device, &render_queue);
}

pub struct DirectionalLight2dPlugin;

impl Plugin for DirectionalLight2dPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DirectionalLight2d>();

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app
            .init_resource::<DirectionalLight2dUniform>()
            .add_systems(RenderStartup, write_initial_directional_light_uniform)
            .add_systems(ExtractSchedule, extract_directional_light);
    }
}

fn write_initial_directional_light_uniform(
    mut uniform: ResMut<DirectionalLight2dUniform>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    uniform
        .buffer
        .write_buffer(&render_device, &render_queue);
}
