struct VSInput {
    // location = 0
    float3 position : POSITION0;
    // location = 1
    float2 texture_coords : TEXCOORD0;
};

struct VSOutput {
    float4 position : SV_Position;
    float2 texture_coords : TEXCOORD0;
};

// ubo
struct GlobalUbo {
    float4x4 projection;
    float4x4 view;
};

// push constant
[[vk::push_constant]]
cbuffer PushConstants {
    float4x4 model;
};

// Bind the uniform buffer, binding=1, set=0
static const int DESCRIPTOR_SET_GLOBAL_UBO = 0;
static const int DESCRIPTOR_BINDING_GLOBAL_UBO = 0;
[[vk::binding(DESCRIPTOR_BINDING_GLOBAL_UBO, DESCRIPTOR_SET_GLOBAL_UBO)]]
ConstantBuffer<GlobalUbo> GLOBAL_UBO;

[shader("vertex")]
VSOutput main(VSInput input) {
    VSOutput output;
    output.position = mul(GLOBAL_UBO.projection, mul(GLOBAL_UBO.view, mul(SLANG_parameterGroup_PushConstants.model, float4(input.position, 1.0))));
    output.texture_coords = input.texture_coords;
    return output;
}