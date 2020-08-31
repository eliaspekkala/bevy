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

        let data: cgltf_data = *out_data;
        let meshes: cgltf_mesh = *data.meshes;
        let primitives: cgltf_primitive = *meshes.primitives;
        let attributes: &mut [cgltf_attribute] = std::slice::from_raw_parts_mut(
            primitives.attributes,
            primitives.attributes_count as usize,
        );

        // Positions
        let positions_accessor: cgltf_accessor = *attributes[0].data;
        let mut positions_flat: Vec<f32> = vec![0.0; (positions_accessor.count * 3) as usize];
        cgltf_accessor_unpack_floats(
            &positions_accessor,
            positions_flat.as_mut_ptr(),
            positions_accessor.count * 3,
        );
        let mut positions: Vec<[f32; 3]> = vec![[0.0; 3]; positions_accessor.count as usize];
        for i in 0..positions_accessor.count as usize {
            positions[i] = [
                positions_flat[i * 3 + 0],
                positions_flat[i * 3 + 1],
                positions_flat[i * 3 + 2],
            ];
        }

        // Normals
        let normals_accessor: cgltf_accessor = *attributes[1].data;
        let mut normals_flat: Vec<f32> = vec![0.0; (normals_accessor.count * 3) as usize];
        cgltf_accessor_unpack_floats(
            &normals_accessor,
            normals_flat.as_mut_ptr(),
            normals_accessor.count * 3,
        );
        let mut normals: Vec<[f32; 3]> = vec![[0.0; 3]; normals_accessor.count as usize];
        for i in 0..normals_accessor.count as usize {
            normals[i] = [
                normals_flat[i * 3 + 0],
                normals_flat[i * 3 + 1],
                normals_flat[i * 3 + 2],
            ];
        }

        // Uvs
        let uvs_accessor: cgltf_accessor = *attributes[2].data;
        let mut uvs_flat: Vec<f32> = vec![0.0; (uvs_accessor.count * 2) as usize];
        cgltf_accessor_unpack_floats(&uvs_accessor, uvs_flat.as_mut_ptr(), uvs_accessor.count * 2);
        let mut uvs: Vec<[f32; 2]> = vec![[0.0; 2]; uvs_accessor.count as usize];
        for i in 0..uvs_accessor.count as usize {
            uvs[i] = [uvs_flat[i * 2 + 0], uvs_flat[i * 2 + 1]];
        }

        // Indices
        let indices_accessor: cgltf_accessor = *primitives.indices;
        let mut indices_flat: Vec<f32> = vec![0.0; indices_accessor.count as usize];
        cgltf_accessor_unpack_floats(
            &indices_accessor,
            indices_flat.as_mut_ptr(),
            indices_accessor.count,
        );
        let mut indices: Vec<u32> = vec![0; indices_accessor.count as usize];
        for i in 0..indices_accessor.count as usize {
            indices[i] = indices_flat[i] as u32;
        }

        // Mesh
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.attributes.push(VertexAttribute::position(positions));
        mesh.attributes.push(VertexAttribute::normal(normals));
        mesh.attributes.push(VertexAttribute::uv(uvs));
        mesh.indices = Some(indices);

        cgltf_free(out_data);

        return mesh;
    }
}
