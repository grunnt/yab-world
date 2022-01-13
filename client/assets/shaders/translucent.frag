#version 330 core

out vec4 fragColor;

uniform sampler2DArray blockTextures;

in VS_OUTPUT {
    vec3 Position;
    vec2 TexCoord;
    flat float Layer;
    vec3 Normal;
    float Light;
} IN;

uniform float Alpha;

uniform mat4 View;

// Lighting
uniform vec3 sunLightDirection;
uniform vec3 sunLightColor;

// Fog
uniform vec3 fogColor;
uniform float fogStart;
uniform float fogEnd;

void main()
{
    vec4 inColor = texture(blockTextures, vec3(IN.TexCoord, IN.Layer));

    // Compute sunlight (diffuse)
    vec3 nSunDirection = normalize((View * vec4(sunLightDirection, 0.0)).xyz);
    float sunIntensity = max(dot(nSunDirection, IN.Normal), 0.0);
    vec3 sunLight = 0.7 * sunIntensity * sunLightColor * inColor.xyz;

     // Calculate ambient light
    vec3 ambientLight = vec3(0.3 * inColor.xyz);

    // Calculate lamp light
    float lampLightLevel = pow(IN.Light, 2);
    vec3 lampLight = inColor.xyz * lampLightLevel;

    // Combine light into final fragment color
    vec3 color = ambientLight + sunLight + lampLight * inColor.xyz;
    
    // Calculate fog
    float fogDistance = length(IN.Position);
    float fogAmount = pow(1.0 - clamp((fogEnd - fogDistance) / (fogEnd - fogStart), 0.0, 1.0), 4);
   
    // Mix fog into color
    fragColor = vec4(mix(color, fogColor, fogAmount), inColor.w);    
}