use bevy_render::{
    mesh::{Mesh, VertexAttribute},
    pipeline::PrimitiveTopology,
};

use anyhow::Result;
use bevy_asset::AssetLoader;
use rgltf::ffi::*;
use std::{ffi::CString, path::Path};

/// Loads meshes from GLTF files into Mesh assets
#[derive(Default)]
pub struct GltfLoader;

impl AssetLoader<Mesh> for GltfLoader {
    fn from_bytes(&self, asset_path: &Path, bytes: Vec<u8>) -> Result<Mesh> {
        let mesh = load_gltf(asset_path, bytes);
        Ok(mesh)
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["gltf", "glb"];
        EXTENSIONS
    }
}

fn load_gltf(asset_path: &Path, _bytes: Vec<u8>) -> Mesh {
    unsafe {
        let path = CString::new(asset_path.as_os_str().to_str().unwrap()).unwrap();
        let path: *const std::os::raw::c_char = path.as_ptr();

        let options: *const cgltf_options =
            std::mem::MaybeUninit::<cgltf_options>::zeroed().as_ptr();

        let mut out_data: *mut cgltf_data = std::ptr::null_mut();

        let mut result: cgltf_result = cgltf_parse_file(options, path, &mut out_data);
        if result != cgltf_result_cgltf_result_success {
            panic!("Failed to parse file: {}", result);
        }

        result = cgltf_load_buffers(options, out_data, path);
        if result != cgltf_result_cgltf_result_success {
            panic!("Failed to load buffers {}", result);
        }

        let data = *out_data;
        let meshes = *data.meshes;
        let primitives = *meshes.primitives;
        let attributes = std::slice::from_raw_parts_mut(
            primitives.attributes,
            primitives.attributes_count as usize,
        );

        // POSITIONS
        let positions_accessor = *attributes[0].data;
        let positions_count = positions_accessor.count;
        let mut positions_out: Vec<[f32; 3]> = Vec::new();
        positions_out.resize(positions_count as usize, [0.0; 3]);
        let positions_count_adj = cgltf_accessor_unpack_floats(
            &positions_accessor,
            std::ptr::null_mut(),
            positions_count,
        );
        let mut positions_temp_out: Vec<f32> = Vec::new();
        positions_temp_out.resize(positions_count_adj as usize, 0.0);
        cgltf_accessor_unpack_floats(
            &positions_accessor,
            positions_temp_out.as_mut_ptr(),
            positions_count_adj as u64,
        );
        for i in 0..3321 {
            positions_out[i] = [
                positions_temp_out[i * 3 + 0],
                positions_temp_out[i * 3 + 1],
                positions_temp_out[i * 3 + 2],
            ];
        }
        let positions: Vec<[f32; 3]> = positions_out;

        // NORMALS
        let normals_accessor = *attributes[1].data;
        let normals_count = normals_accessor.count;
        let mut normals_out: Vec<[f32; 3]> = Vec::new();
        normals_out.resize(normals_count as usize, [0.0; 3]);
        let normals_count_adj =
            cgltf_accessor_unpack_floats(&normals_accessor, std::ptr::null_mut(), normals_count);
        let mut normals_temp_out: Vec<f32> = Vec::new();
        normals_temp_out.resize(normals_count_adj as usize, 0.0);
        cgltf_accessor_unpack_floats(
            &normals_accessor,
            normals_temp_out.as_mut_ptr(),
            normals_count_adj as u64,
        );
        for i in 0..3321 {
            normals_out[i] = [
                normals_temp_out[i * 3 + 0],
                normals_temp_out[i * 3 + 1],
                normals_temp_out[i * 3 + 2],
            ];
        }
        let normals: Vec<[f32; 3]> = normals_out;

        // UVS
        let uvs_accessor = *attributes[2].data;
        let uvs_count = uvs_accessor.count;
        let mut uvs_out: Vec<[f32; 2]> = Vec::new();
        uvs_out.resize(uvs_count as usize, [0.0; 2]);
        let uvs_count_adj =
            cgltf_accessor_unpack_floats(&uvs_accessor, std::ptr::null_mut(), uvs_count);
        let mut uvs_temp_out: Vec<f32> = Vec::new();
        uvs_temp_out.resize(uvs_count_adj as usize, 0.0);
        cgltf_accessor_unpack_floats(
            &uvs_accessor,
            uvs_temp_out.as_mut_ptr(),
            uvs_count_adj as u64,
        );
        for i in 0..3321 {
            uvs_out[i] = [uvs_temp_out[i * 2 + 0], uvs_temp_out[i * 2 + 1]];
        }
        let uvs: Vec<[f32; 2]> = uvs_out;

        // INDICIES
        let indices_accessor = *primitives.indices;
        let indices_count = indices_accessor.count;
        let mut indices_out: Vec<f32> = Vec::new();
        indices_out.resize(indices_count as usize, 0.0);
        let indices_count_adj =
            cgltf_accessor_unpack_floats(&indices_accessor, std::ptr::null_mut(), indices_count);
        cgltf_accessor_unpack_floats(
            &indices_accessor,
            indices_out.as_mut_ptr(),
            indices_count_adj as u64,
        );
        let indices: Vec<u32> = indices_out.into_iter().map(|i| i as u32).collect();

        // DEBUG
        // println!("positions_count: {:#?} \n", positions_count); // 3321
        // println!("normals_count: {:#?} \n", normals_count); // 3321
        // println!("uvs_count: {:#?} \n", uvs_count); // 3321
        // println!("indices_count: {:#?} \n", indices_count); // 11808

        // println!("positions_count_adj: {:#?} \n", positions_count_adj); // 9963
        // println!("normals_count_adj: {:#?} \n", normals_count_adj); // 9963
        // println!("uvs_count_adj: {:#?} \n", uvs_count_adj); // 6642
        // println!("indices_count_adj: {:#?} \n", indices_count_adj); // 11808

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.attributes.push(VertexAttribute::position(positions));
        mesh.attributes.push(VertexAttribute::normal(normals));
        mesh.attributes.push(VertexAttribute::uv(uvs));
        mesh.indices = Some(indices);

        cgltf_free(out_data);

        return mesh;
    }
}
