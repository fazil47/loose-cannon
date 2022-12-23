// The clouds are below the planet, they are yellow and they are moving like a wave

use bevy::{
    prelude::{
        default, AlphaMode, Assets, Commands, Material, MaterialMeshBundle, Mesh, Name, Res,
        ResMut, Transform, Vec2, Vec3,
    },
    reflect::TypeUuid,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef},
    },
    time::Time,
};
use itertools::Itertools;

// CONSTANTS

const CLOUD_ALTITUDE: f32 = -70.0;

// STARTUP SYSTEMS

pub fn setup_clouds(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<CloudMaterial>>,
) {
    let mut cloud = Mesh::from(Cloud {
        size: 320.0,
        num_vertices: 64,
    });

    if let Some(VertexAttributeValues::Float32x3(positions)) =
        cloud.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let colors: Vec<[f32; 4]> = positions
            .iter()
            .map(|[_r, _g, _b]| [1.0, 1.0, 0.2, 1.0])
            .collect();

        cloud.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }

    commands
        .spawn(MaterialMeshBundle {
            mesh: meshes.add(cloud),
            transform: Transform::from_xyz(0.0, CLOUD_ALTITUDE, 0.0)
                .with_scale(Vec3::new(10.0, 1.0, 10.0)),
            material: materials.add(CloudMaterial {
                time: 0.0,
                steepness: 0.25,
                wavelength: 50.0,
                speed: 10.0,
                wave_1_dir: Vec2::new(1.0, 1.0),
                wave_2_dir: Vec2::new(1.0, 0.6),
                wave_3_dir: Vec2::new(1.3, -0.1),
                alpha_mode: AlphaMode::Blend,
            }),
            ..default()
        })
        .insert(Name::new("Clouds"));
}

// SYSTEMS

pub fn update_clouds(time: Res<Time>, mut materials: ResMut<Assets<CloudMaterial>>) {
    for material in materials.iter_mut() {
        material.1.time = time.elapsed_seconds_wrapped();
    }
}

// MATERIALS

/// The Material trait is very configurable, but comes with sensible defaults for all methods.
/// You only need to implement functions for features that need non-default behavior. See the Material api docs for details!
impl Material for CloudMaterial {
    // fn fragment_shader() -> ShaderRef {
    //     "shaders/custom_material.wgsl".into()
    // }

    fn vertex_shader() -> ShaderRef {
        "shaders/cloud_vertex.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}

// This is the struct that will be passed to your shader
#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CloudMaterial {
    #[uniform(0)]
    time: f32,
    #[uniform(1)]
    steepness: f32,
    #[uniform(2)]
    wavelength: f32,
    #[uniform(3)]
    speed: f32,
    #[uniform(4)]
    wave_1_dir: Vec2,
    #[uniform(5)]
    wave_2_dir: Vec2,
    #[uniform(6)]
    wave_3_dir: Vec2,
    alpha_mode: AlphaMode,
}

#[derive(Debug, Copy, Clone)]
struct Cloud {
    size: f32,
    num_vertices: u32,
}

impl From<Cloud> for Mesh {
    fn from(plane: Cloud) -> Self {
        let extent = plane.size / 2.0;

        let jump = extent / plane.num_vertices as f32;

        let vertices = (0..=plane.num_vertices)
            .cartesian_product(0..=plane.num_vertices)
            .map(|(y, x)| {
                (
                    [
                        x as f32 * jump - 0.5 * extent,
                        0.0,
                        y as f32 * jump - 0.5 * extent,
                    ],
                    [0.0, 1.0, 0.0],
                    [
                        x as f32 / plane.num_vertices as f32,
                        y as f32 / plane.num_vertices as f32,
                    ],
                )
            })
            .collect::<Vec<_>>();

        let indices = Indices::U32(
            (0..=plane.num_vertices)
                .cartesian_product(0..=plane.num_vertices)
                .enumerate()
                .filter_map(|(index, (x, y))| {
                    if y >= plane.num_vertices {
                        None
                    } else if x >= plane.num_vertices {
                        None
                    } else {
                        Some([
                            [
                                index as u32,
                                index as u32 + 1 + 1 + plane.num_vertices,
                                index as u32 + 1,
                            ],
                            [
                                index as u32,
                                index as u32 + 1 + plane.num_vertices,
                                index as u32 + plane.num_vertices + 1 + 1,
                            ],
                        ])
                    }
                })
                .flatten()
                .flatten()
                .collect::<Vec<_>>(),
        );

        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
