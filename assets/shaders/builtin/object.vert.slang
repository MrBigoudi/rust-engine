// location = 0
struct VSInput {
    float3 position : POSITION0;
};

struct VSOutput {
    float4 position : SV_Position;
};

struct GlobalUniformObject {
    float4x4 projection;
    float4x4 view;
};

// Bind the uniform buffer, binding=0, set=0
ConstantBuffer<GlobalUniformObject> global_ubo : register(b0, space0);

[shader("vertex")]
VSOutput main(VSInput input) {
    VSOutput output;
    output.position = mul(global_ubo.projection, mul(global_ubo.view, float4(input.position, 1.0)));
    return output;
}