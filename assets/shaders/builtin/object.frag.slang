// location = 0
struct FSInput {
    float4 position : SV_Position;
    float2 texture_coords : TEXCOORD0;
};

// location = 0
struct FSOutput {
    float4 color : SV_Target0;
};

// ubo
struct PerObjectUbo {
    float4 diffuse_color;
};

// Bind the uniform buffer, binding=0, set=1
static const int DESCRIPTOR_SET_PER_OBJECT_UBO = 1;
static const int DESCRIPTOR_BINDING_PER_OBJECT_UBO = 0;
[[vk::binding(DESCRIPTOR_BINDING_PER_OBJECT_UBO, DESCRIPTOR_SET_PER_OBJECT_UBO)]]
ConstantBuffer<PerObjectUbo> PER_OBJECT_UBO;

// Combined image sampler (Texture + Sampler), binding=1, set=1
static const int DESCRIPTOR_SET_DIFFUSE_SAMPLER = 1;
static const int DESCRIPTOR_BINDING_DIFFUSE_SAMPLER = 1;
[[vk::binding(DESCRIPTOR_BINDING_DIFFUSE_SAMPLER, DESCRIPTOR_SET_DIFFUSE_SAMPLER)]]
Sampler2D DIFFUSE_SAMPLER;

[shader("fragment")]
FSOutput main(FSInput input) {
    FSOutput output;

    // Sample the texture using the combined image sampler and texture coordinates
    float4 sampledColor = DIFFUSE_SAMPLER.Sample(input.texture_coords);

    // Modulate the sampled color by the diffuse color
    output.color = PER_OBJECT_UBO.diffuse_color * sampledColor;
    
    return output;
}