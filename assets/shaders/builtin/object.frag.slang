struct FSOutput {
    float4 colour : SV_Target0;
};

[shader("fragment")]
FSOutput main() {
    FSOutput output;
    output.colour = float4(1.0, 1.0, 1.0, 1.0);
    return output;
}