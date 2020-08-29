use bevy_render::{
    mesh::{Mesh, VertexAttribute},
    pipeline::PrimitiveTopology,
};

use anyhow::Result;
use bevy_asset::AssetLoader;
use gltf::buffer::Source;
use rgltf::ffi;
use std::{fs, path::Path};

/// Loads meshes from GLTF files into Mesh assets
///
/// NOTE: eventually this will loading into Scenes instead of Meshes
#[derive(Default)]
pub struct GltfLoader;

impl AssetLoader<Mesh> for GltfLoader {
    fn from_bytes(&self, asset_path: &Path, bytes: Vec<u8>) -> Result<Mesh> {
        let mesh = load_rgltf(asset_path, bytes);
        // let mesh = load_gltf(asset_path, bytes);
        Ok(mesh)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["gltf", "glb"];
        EXTENSIONS
    }
}

fn load_rgltf(_path: &Path, _bytes: Vec<u8>) -> Mesh {
    unsafe {
        let path = std::ffi::CString::new("./assets/models/monkey/Monkey.gltf").unwrap();
        let path: *const std::os::raw::c_char = path.as_ptr();

        let options: *const ffi::cgltf_options =
            std::mem::MaybeUninit::<ffi::cgltf_options>::zeroed().as_ptr();

        let mut out_data: *mut ffi::cgltf_data = std::ptr::null_mut();

        let mut result: ffi::cgltf_result = ffi::cgltf_parse_file(options, path, &mut out_data);
        if result != ffi::cgltf_result_cgltf_result_success {
            panic!("Failed to load file: {}", result);
        }

        result = ffi::cgltf_load_buffers(options, out_data, path);
        if result != ffi::cgltf_result_cgltf_result_success {
            panic!("Failed to load buffers {}", result);
        }

        let data = *out_data;
        let meshes = *data.meshes;
        let primitives = *meshes.primitives;
        let attributes = *primitives.attributes;

        // println!("data: {:#?} \n", data);
        // println!("meshes: {:#?} \n", meshes);
        // println!("primitives: {:#?} \n", primitives);
        // println!("attributes: {:#?} \n", attributes);

        let positions_accessor = *attributes.data;
        let indices_accessor = *primitives.indices;

        let positions_count = positions_accessor.count;
        let indices_count = indices_accessor.count;

        // println!("positions_accessor: {:#?} \n", positions_accessor);
        // println!("indices_accessor: {:#?} \n", indices_accessor);

        // println!("positions_count: {:#?} \n", positions_count); // 3321
        // println!("indices_count: {:#?} \n", indices_count); // 11808

        // Positions is cgltf_type_vec3.
        // Indicies is cgltf_type_scalar.

        // let positions_out: *mut ffi::cgltf_float = std::ptr::null_mut();
        // let indices_out: *mut ffi::cgltf_float = std::ptr::null_mut();
        // let positions_count =
        //     ffi::cgltf_accessor_unpack_floats(&positions_accessor, positions_out, positions_count);
        // let indices_count =
        //     ffi::cgltf_accessor_unpack_floats(&indices_accessor, indices_out, indices_count);
        // println!("positions_count: {:#?} \n", positions_count); // 9963
        // println!("indices_count: {:#?} \n", indices_count); // 11808

        let mut positions_out: Vec<[f32; 3]> = Vec::new();
        positions_out.resize(3321, [0.0; 3]);

        let mut positions_temp_out: Vec<f32> = Vec::new();
        positions_temp_out.resize(9963, 0.0);

        ffi::cgltf_accessor_unpack_floats(
            &positions_accessor,
            positions_temp_out.as_mut_ptr(),
            positions_out.len() as u64,
        );

        for i in 0..3321 {
            for j in 0..3 {
                positions_out[i][j] = positions_temp_out[i * 3 + j];
            }
        }

        // println!("positions_out: {:#?} \n", positions_out[0]);

        let mut indices_out: Vec<f32> = Vec::new();
        indices_out.resize(11808, 0.0);
        ffi::cgltf_accessor_unpack_floats(
            &indices_accessor,
            indices_out.as_mut_ptr(),
            indices_out.len() as u64,
        );

        // println!("indices_out: {:#?} \n", indices_out);

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let positions: Vec<[f32; 3]> = positions_out;
        let normals: Vec<[f32; 3]> = vec![[0.0; 3]; 3321];
        let uvs: Vec<[f32; 2]> = vec![[0.0; 2]; 3321];
        let indices: Vec<u32> = indices_out.into_iter().map(|i| i as u32).collect();

        mesh.attributes.push(VertexAttribute::position(positions));
        mesh.attributes.push(VertexAttribute::normal(normals));
        mesh.attributes.push(VertexAttribute::uv(uvs));
        mesh.indices = Some(indices);

        ffi::cgltf_free(out_data);

        return mesh;
    }
}

pub fn load_gltf(asset_path: &Path, bytes: Vec<u8>) -> Mesh {
    let gltf = gltf::Gltf::from_slice(&bytes).unwrap();

    let buffer_data = load_buffers(&gltf, asset_path);

    if let Some(node) = gltf.nodes().next() {
        if let Some(mesh) = node.mesh() {
            if let Some(primitive) = mesh.primitives().next() {
                let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
                let primitive_topology = PrimitiveTopology::TriangleList;
                let mut mesh = Mesh::new(primitive_topology);

                if let Some(vertex_attribute) = reader
                    .read_positions()
                    .map(|v| VertexAttribute::position(v.collect()))
                {
                    mesh.attributes.push(vertex_attribute);
                }

                if let Some(vertex_attribute) = reader
                    .read_normals()
                    .map(|v| VertexAttribute::normal(v.collect()))
                {
                    mesh.attributes.push(vertex_attribute);
                }

                if let Some(vertex_attribute) = reader
                    .read_tex_coords(0)
                    .map(|v| VertexAttribute::uv(v.into_f32().collect()))
                {
                    mesh.attributes.push(vertex_attribute);
                }

                if let Some(indices) = reader.read_indices() {
                    mesh.indices = Some(indices.into_u32().collect::<Vec<u32>>());
                };

                return mesh;
            }
        }
    }
    panic!("No mesh found!")
}

fn load_buffers(gltf: &gltf::Gltf, asset_path: &Path) -> Vec<Vec<u8>> {
    let mut buffer_data: Vec<Vec<u8>> = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            Source::Uri(uri) => {
                let buffer_path = asset_path.parent().unwrap().join(uri);
                let buffer_bytes = fs::read(buffer_path).unwrap();
                buffer_data.push(buffer_bytes);
            }
            Source::Bin => {
                let blob = gltf.blob.as_deref().unwrap();
                buffer_data.push(blob.to_vec());
            }
        }
    }
    buffer_data
}
