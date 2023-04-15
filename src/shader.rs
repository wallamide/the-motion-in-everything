#![allow(clippy::extra_unused_type_parameters)]
use crate::file_system_interaction::asset_loading::TextureAssets;
use crate::GameState;
use anyhow::{Context, Result};
use bevy::{
    prelude::*,
    asset::HandleId,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    reflect::TypeUuid,
    render::{
        extract_resource::{ExtractResourcePlugin, ExtractResource},
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, Face::Front, ShaderRef, ShaderType, SpecializedMeshPipelineError, TextureDimension, TextureFormat, Extent3d, TextureUsages, BindGroup, BindGroupLayout, CachedComputePipelineId, BindGroupLayoutDescriptor, BindGroupLayoutEntry, ShaderStages, StorageTextureAccess, PipelineCache, BindingType, TextureViewDimension, ComputePipelineDescriptor, BindGroupEntry, BindingResource, BindGroupDescriptor,
        },
        renderer::RenderDevice,
        Render, RenderApp, RenderSet, render_graph::RenderGraph,
    },
    sprite::Material2d,
    utils::HashMap,
};
use bevy_mod_sysfail::macros::*;
use bevy_rapier3d::pipeline;
use regex::Regex;
use std::borrow::Cow;
use std::sync::LazyLock;

pub struct ShaderPlugin;

/// Handles instantiation of shaders. The shaders can be found in the [`shaders`](https://github.com/janhohenheim/foxtrot/tree/main/assets/shaders) directory.
/// Shaders are stored in [`Material`]s which can be used on objects by attaching a `Handle<Material>` to an entity.
/// The handles can be stored and retrieved in the [`Materials`] resource.
impl Plugin for ShaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<GlowyMaterial>::default())
        .add_plugin(MaterialPlugin::<RepeatedMaterial>::default())
        .add_plugin(MaterialPlugin::<SkydomeMaterial>::default())
        .add_plugin(ExtractResourcePlugin::<KuwaharaImage>::default()) // ToDO: change to compute shader
        .add_system(setup_shader.in_schedule(OnExit(GameState::Loading)))
        .add_system(set_texture_to_repeat.in_set(OnUpdate(GameState::Playing)))
        .sub_app_mut(RenderApp)
            .init_resource::<KuwaharaPipeline>()
            .add_systems(Render, queue_bind_group.in_set(RenderSet::Queue));

        let mut render_graph = app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("kuwahara_filter", KuwaharaNode::default());
        render_graph.add_node_edge(
            "kuwahara_filter",
            bevy::render::main_graph::node::CAMERA_DRIVER, 
        );
    }
}

#[derive(Resource, Debug, Clone)]
pub struct Materials {
    pub glowy: Handle<GlowyMaterial>,
    /// (Texture asset ID, Repeats) -> RepeatedMaterial
    pub repeated: HashMap<(HandleId, Repeats), Handle<RepeatedMaterial>>,
    pub skydome: Handle<SkydomeMaterial>,
    // pub kuwahara: Handle<KuwaharaMaterial>,
}

const SIZE: (u32, u32) = (1280,720);
const WORKGROUP_SIZE: u32 = 8;

fn setup_shader(
    mut commands: Commands,
    mut glow_materials: ResMut<Assets<GlowyMaterial>>,
    mut skydome_materials: ResMut<Assets<SkydomeMaterial>>,
    // mut kuwahara_materials: ResMut<Assets<KuwaharaMaterial>>,
    texture_assets: Res<TextureAssets>,
    mut images: ResMut<Assets<Image>>,
) {
    let glowy = glow_materials.add(GlowyMaterial {
        env_texture: texture_assets.glowy_interior.clone(),
    });
    let skydome = skydome_materials.add(SkydomeMaterial {
        env_texture: texture_assets.sky.clone(),
    });

    // Taken from bevy compute_shader_game_of_life example.

    let mut image = Image::new_fill(
        Extent3d {
            width: SIZE.0,
            height: SIZE.1,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8Unorm,
    );
    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;
    let image = images.add(image);

    commands.spawn(SpriteBundle {
        sprite: Sprite { 
            custom_size: Some(Vec2::new(SIZE.0 as f32, SIZE.1 as f32)),
            ..default()
        },
        texture: image.clone(),
        ..default()
    });
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(KuwaharaImage(image));

    commands.insert_resource(Materials {
        repeated: HashMap::new(),
        glowy,
        skydome,
        // kuwahara,
    });
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "bd5c76fd-6fdd-4de4-9744-4e8beea8daaf"]
/// Material for [`glowy.wgsl`](https://github.com/janhohenheim/foxtrot/blob/main/assets/shaders/glowy.wgsl).
pub struct GlowyMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub env_texture: Handle<Image>,
}

impl Material for GlowyMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/glowy.wgsl".into()
    }
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "8ca95d76-91d6-44c0-a67b-8a4d22cd59b1"]
/// Material for [`skydome.wgsl`](https://github.com/janhohenheim/foxtrot/blob/main/assets/shaders/skydome.wgsl).
pub struct SkydomeMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub env_texture: Handle<Image>,
}

impl Material for SkydomeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/skydome.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Front);
        Ok(())
    }
}

#[derive(Resource, Clone, Deref, ExtractResource)]
// Material for the kuwahara filter
struct KuwaharaImage (
    // #[texture(0)]
    // #[sampler(1)]
    Handle<Image>
);

#[derive(Resource)]
struct KuwaharaImageBindGroup(BindGroup);

fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<KuwaharaPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    kuwahara_image: Res<KuwaharaImage>,
    render_device: Res<RenderDevice>,
){
    let view = &gpu_images[&kuwahara_image.0];
    let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &pipeline.texture_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: BindingResource::TextureView(&view.texture_view),
        }],
    });
    commands.insert_resource(KuwaharaImageBindGroup(bind_group));
}

#[derive(Resource)]
pub struct KuwaharaPipeline {
    texture_bind_group_layout: BindGroupLayout,
    init_pipeline: CachedComputePipelineId,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for KuwaharaPipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
            .resource::<RenderDevice>()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: None,
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                }],
            });
        let shader = world
                .resource::<AssetServer>()
                .load("shaders/kuwahara_filter_compute.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let init_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor{
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("init"),
        });
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader,
            shader_defs: vec![],
            entry_point: Cow::from("update"),
        });

        KuwaharaPipeline {
            texture_bind_group_layout,
            init_pipeline,
            update_pipeline,
        }
    }
}

struct KuwaharaNode;

#[repr(C, align(16))] // All WebGPU uniforms must be aligned to 16 bytes
#[derive(Clone, Copy, ShaderType, Debug, Hash, Eq, PartialEq, Default)]
pub struct Repeats {
    pub horizontal: u32,
    pub vertical: u32,
    pub _wasm_padding1: u32,
    pub _wasm_padding2: u32,
}

#[derive(AsBindGroup, Debug, Clone, TypeUuid)]
#[uuid = "82d336c5-fd6c-41a3-bdd4-267cd4c9be22"]
/// Material for [`repeated.wgsl`](https://github.com/janhohenheim/foxtrot/blob/main/assets/shaders/repeated.wgsl).
pub struct RepeatedMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub texture: Handle<Image>,
    #[uniform(2)]
    pub repeats: Repeats,
}

impl Material for RepeatedMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/repeated.wgsl".into()
    }
}

static REPEAT_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\[repeat:\s*(\d+),\s*(\d+)\]").expect("Failed to compile repeat regex")
});

#[sysfail(log(level = "error"))]
pub fn set_texture_to_repeat(
    mut commands: Commands,
    added_name: Query<(&Name, &Children), Added<Name>>,
    material_handles: Query<&Handle<StandardMaterial>>,
    mut materials: ResMut<Materials>,
    standard_materials: Res<Assets<StandardMaterial>>,
    mut repeated_materials: ResMut<Assets<RepeatedMaterial>>,
) -> Result<()> {
    for (name, children) in &added_name {
        if let Some(captures) = REPEAT_REGEX.captures(&name.to_lowercase()) {
            let repeats = Repeats {
                horizontal: captures[1].parse().context("Failed to parse repeat")?,
                vertical: captures[2].parse().context("Failed to parse repeat")?,
                ..default()
            };
            for child in children.iter() {
                if let Ok(standard_material_handle) = material_handles.get(*child) {
                    let standard_material = standard_materials
                        .get(standard_material_handle)
                        .context("Failed to get standard material from handle")?;
                    let texture = standard_material.base_color_texture.as_ref().context(
                        "Failed to get texture from standard material. Is the texture missing?",
                    )?;
                    let key = (texture.id(), repeats);

                    let repeated_material = materials.repeated.entry(key).or_insert_with(|| {
                        repeated_materials.add(RepeatedMaterial {
                            texture: texture.clone(),
                            repeats,
                        })
                    });

                    commands
                        .entity(*child)
                        .remove::<Handle<StandardMaterial>>()
                        .insert(repeated_material.clone());
                }
            }
        }
    }
    Ok(())
}
