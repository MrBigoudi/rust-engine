struct VSInput {
    float3 position : POSITION;
};

struct VSOutput {
    float4 position : SV_Position;
};

[shader("vertex")]
VSOutput main(VSInput input) {
    VSOutput output;
    output.position = float4(input.position, 1.0);
    return output;
}