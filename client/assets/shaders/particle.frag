#version 330 core

uniform sampler2DArray textures;

out vec4 fragColor;

flat in float layer;
flat in float life;

void main()
{
    vec4 texColor = texture(textures, vec3(gl_PointCoord, layer));
    if(texColor.a < 0.1) {
        discard;
    }
    fragColor = vec4(texColor.rgb, texColor.a * smoothstep(0.0, 0.5, life));
}