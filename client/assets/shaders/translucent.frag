#version 330 core

out vec4 fragColor;

uniform sampler2DArray blockTextures;

in VS_OUTPUT {
    vec3 Position;
    vec2 TexCoord;
    flat float Layer;
    vec3 Normal;
    vec2 Light;
} IN;

uniform float Alpha;

uniform mat4 View;

// Lighting
uniform vec3 ambientLightColor;
uniform vec3 sunLightDirection;
uniform vec3 sunLightColor;

// Fog
uniform vec3 fogColor;
uniform float fogStart;
uniform float fogEnd;

void main()
{
    vec3 inColor = texture(blockTextures, vec3(IN.TexCoord, IN.Layer)).xyz;

      // Compute sunlight (diffuse)
    float sunLightLevel = IN.Light.g;
    vec3 nSunDirection = normalize((View * vec4(sunLightDirection, 0.0)).xyz);
    float sunIntensity = max(dot(nSunDirection, IN.Normal), 0.0) * sunLightLevel;
    vec3 sunColor = sunIntensity * sunLightColor;
 
    // Combine ambient and sunlight
    float lightLevel = pow(IN.Light.r, 2);
    vec3 color = (ambientLightColor + sunColor + lightLevel) * inColor;
    
    // Calculate fog
    float fogDistance = length(IN.Position);
    float fogAmount = pow(1.0 - clamp((fogEnd - fogDistance) / (fogEnd - fogStart), 0.0, 1.0), 2);
   
    // Mix fog into color
    fragColor = vec4(mix(color, fogColor, fogAmount), Alpha);    
}