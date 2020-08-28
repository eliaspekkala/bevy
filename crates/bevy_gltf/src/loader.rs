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
        let indicies_accessor = *primitives.indices;

        let positions_count = positions_accessor.count;
        let indices_count = indicies_accessor.count;

        let positions_offset = positions_accessor.offset;
        let indices_offset = indicies_accessor.offset;

        let positions_stride = positions_accessor.stride;
        let indices_stride = indicies_accessor.stride;

        let positions_buffer_view = *positions_accessor.buffer_view;
        let indicies_buffer_view = *indicies_accessor.buffer_view;

        let positions_buffer = *positions_buffer_view.buffer;
        let indicies_buffer = *indicies_buffer_view.buffer;

        let positions_data = positions_buffer.data;
        let indicies_data = indicies_buffer.data;

        println!("positions_accessor: {:#?} \n", positions_accessor);
        println!("indicies_accessor: {:#?} \n", indicies_accessor);

        println!("positions_count: {:#?} \n", positions_count);
        println!("indices_count: {:#?} \n", indices_count);

        println!("positions_offset: {:#?} \n", positions_offset);
        println!("indices_offset: {:#?} \n", indices_offset);

        println!("positions_stride: {:#?} \n", positions_stride);
        println!("indices_stride: {:#?} \n", indices_stride);

        println!("positions_buffer_view: {:#?} \n", positions_buffer_view);
        println!("indicies_buffer_view: {:#?} \n", indicies_buffer_view);

        println!("positions_buffer: {:#?} \n", positions_buffer);
        println!("indicies_buffer: {:#?} \n", indicies_buffer);

        println!("positions_data: {:#?} \n", *positions_data); // c_void?
        println!("indicies_data: {:#?} \n", *indicies_data); // c_void?

        // TODO: Use one of these functions?
        // ffi::cgltf_accessor_unpack_floats()
        // ffi::cgltf_accessor_num_components()
        // ffi::cgltf_accessor_read_float()
        // ffi::cgltf_accessor_read_uint()
        // ffi::cgltf_accessor_read_index()

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let positions = Vec::<[f32; 3]>::new();
        let indices = Vec::<u32>::new();

        mesh.attributes.push(VertexAttribute::position(positions));
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
