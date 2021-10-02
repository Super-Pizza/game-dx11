struct PixelShaderInput {
    float4 color: COLOR;
};
float4 ShaderMain(PixelShaderInput IN): SV_TARGET {
    return IN.color;
}
