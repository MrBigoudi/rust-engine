use ash::vk::PhysicalDeviceFeatures;

pub(crate) fn physical_device_features_to_vector(
    features: &PhysicalDeviceFeatures,
) -> Vec<(String, bool)> {
    vec![
        (
            String::from("Robust Buffer Access"),
            (features.robust_buffer_access == ash::vk::TRUE),
        ),
        (
            String::from("Full Draw Index Uint32"),
            (features.full_draw_index_uint32 == ash::vk::TRUE),
        ),
        (
            String::from("Image Cube Array"),
            (features.image_cube_array == ash::vk::TRUE),
        ),
        (
            String::from("Independent Blend"),
            (features.independent_blend == ash::vk::TRUE),
        ),
        (
            String::from("Geometry Shader"),
            (features.geometry_shader == ash::vk::TRUE),
        ),
        (
            String::from("Tessellation Shader"),
            (features.tessellation_shader == ash::vk::TRUE),
        ),
        (
            String::from("Sample Rate Shading"),
            (features.sample_rate_shading == ash::vk::TRUE),
        ),
        (
            String::from("Dual Src Blend"),
            (features.dual_src_blend == ash::vk::TRUE),
        ),
        (
            String::from("Logic Op"),
            (features.logic_op == ash::vk::TRUE),
        ),
        (
            String::from("Multi Draw Indirect"),
            (features.multi_draw_indirect == ash::vk::TRUE),
        ),
        (
            String::from("Draw Indirect First Instance"),
            (features.draw_indirect_first_instance == ash::vk::TRUE),
        ),
        (
            String::from("Depth Clamp"),
            (features.depth_clamp == ash::vk::TRUE),
        ),
        (
            String::from("Depth Bias Clamp"),
            (features.depth_bias_clamp == ash::vk::TRUE),
        ),
        (
            String::from("Fill Mode Non Solid"),
            (features.fill_mode_non_solid == ash::vk::TRUE),
        ),
        (
            String::from("Depth Bounds"),
            (features.depth_bounds == ash::vk::TRUE),
        ),
        (
            String::from("Wide Lines"),
            (features.wide_lines == ash::vk::TRUE),
        ),
        (
            String::from("Large Points"),
            (features.large_points == ash::vk::TRUE),
        ),
        (
            String::from("Alpha To One"),
            (features.alpha_to_one == ash::vk::TRUE),
        ),
        (
            String::from("Multi Viewport"),
            (features.multi_viewport == ash::vk::TRUE),
        ),
        (
            String::from("Sampler Anisotropy"),
            (features.sampler_anisotropy == ash::vk::TRUE),
        ),
        (
            String::from("Texture Compresion Etc2"),
            (features.texture_compression_etc2 == ash::vk::TRUE),
        ),
        (
            String::from("Texture Compression Astc Ldr"),
            (features.texture_compression_astc_ldr == ash::vk::TRUE),
        ),
        (
            String::from("Texture Compression Bc"),
            (features.texture_compression_bc == ash::vk::TRUE),
        ),
        (
            String::from("Occlusion Query Precise"),
            (features.occlusion_query_precise == ash::vk::TRUE),
        ),
        (
            String::from("Pipeline Statistics Query"),
            (features.pipeline_statistics_query == ash::vk::TRUE),
        ),
        (
            String::from("Vertex Pipeline Stores And Atomics"),
            (features.vertex_pipeline_stores_and_atomics == ash::vk::TRUE),
        ),
        (
            String::from("Fragment Stores And Atomics"),
            (features.fragment_stores_and_atomics == ash::vk::TRUE),
        ),
        (
            String::from("Shader Tessellation And Geometry Point Size"),
            (features.shader_tessellation_and_geometry_point_size == ash::vk::TRUE),
        ),
        (
            String::from("Shader Image Gather Extended"),
            (features.shader_image_gather_extended == ash::vk::TRUE),
        ),
        (
            String::from("Shader Storage Image Extended Formats"),
            (features.shader_storage_image_extended_formats == ash::vk::TRUE),
        ),
        (
            String::from("Shader Storage Image Multisample"),
            (features.shader_storage_image_multisample == ash::vk::TRUE),
        ),
        (
            String::from("Shader Storage Image Read Without Format"),
            (features.shader_storage_image_read_without_format == ash::vk::TRUE),
        ),
        (
            String::from("Shader Storage Image Write Without Format"),
            (features.shader_storage_image_write_without_format == ash::vk::TRUE),
        ),
        (
            String::from("Shader Uniform Buffer Array Dynamic Indexing"),
            (features.shader_uniform_buffer_array_dynamic_indexing == ash::vk::TRUE),
        ),
        (
            String::from("Shader Sampled Image Array Dynamic Indexing"),
            (features.shader_sampled_image_array_dynamic_indexing == ash::vk::TRUE),
        ),
        (
            String::from("Shader Storage Buffer Array Dynamic Indexing"),
            (features.shader_storage_buffer_array_dynamic_indexing == ash::vk::TRUE),
        ),
        (
            String::from("Shader Storage Image Array Dynamic Indexing"),
            (features.shader_storage_image_array_dynamic_indexing == ash::vk::TRUE),
        ),
        (
            String::from("Shader Clip Distance"),
            (features.shader_clip_distance == ash::vk::TRUE),
        ),
        (
            String::from("Shader Cull Distance"),
            (features.shader_cull_distance == ash::vk::TRUE),
        ),
        (
            String::from("Shader Float64"),
            (features.shader_float64 == ash::vk::TRUE),
        ),
        (
            String::from("Shader Int64"),
            (features.shader_int64 == ash::vk::TRUE),
        ),
        (
            String::from("Shader Int16"),
            (features.shader_int16 == ash::vk::TRUE),
        ),
        (
            String::from("Shader Resource Residency"),
            (features.shader_resource_residency == ash::vk::TRUE),
        ),
        (
            String::from("Shader Resource Min Lod"),
            (features.shader_resource_min_lod == ash::vk::TRUE),
        ),
        (
            String::from("Sparse Binding"),
            (features.sparse_binding == ash::vk::TRUE),
        ),
        (
            String::from("Sparse Residency Buffer"),
            (features.sparse_residency_buffer == ash::vk::TRUE),
        ),
        (
            String::from("Sparse Residency Image2D"),
            (features.sparse_residency_image2_d == ash::vk::TRUE),
        ),
        (
            String::from("Sparse Residency Image3D"),
            (features.sparse_residency_image3_d == ash::vk::TRUE),
        ),
        (
            String::from("Sparse Residency2 Samples"),
            (features.sparse_residency2_samples == ash::vk::TRUE),
        ),
        (
            String::from("Sparse Residency4 Samples"),
            (features.sparse_residency4_samples == ash::vk::TRUE),
        ),
        (
            String::from("Sparse Residency8 Samples"),
            (features.sparse_residency8_samples == ash::vk::TRUE),
        ),
        (
            String::from("Sparse Residency16 Samples"),
            (features.sparse_residency16_samples == ash::vk::TRUE),
        ),
        (
            String::from("Sparse Residency Aliased"),
            (features.sparse_residency_aliased == ash::vk::TRUE),
        ),
        (
            String::from("Variable Multisample Rate"),
            (features.variable_multisample_rate == ash::vk::TRUE),
        ),
        (
            String::from("Inherited Queries"),
            (features.inherited_queries == ash::vk::TRUE),
        ),
    ]
}
