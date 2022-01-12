#version 330 core
out vec4 Color;

in VS_OUTPUT {
    vec2 TexCoords;
} IN;

// Output from stage 1
uniform sampler2D gPosition;
uniform sampler2D gColor;
uniform sampler2D gNormal;
uniform sampler2D gLight; 
uniform sampler2D gRandom; 

// SSAO
const int SSAO_KERNEL_SIZE = 64;
uniform vec3 ssaoKernel[SSAO_KERNEL_SIZE];
const float ssaoSampleRad = 0.5; 
const float ssaoBias = 0.15;
uniform vec2 ssaoTextureScale;

// Projection
uniform mat4 View;
uniform mat4 Projection;

// Lighting
uniform vec3 ambientLightColor;
uniform vec3 sunLightDirection;
uniform vec3 sunLightColor;

// Fog
uniform vec3 fogColor;
uniform float fogStart;
uniform float fogEnd;

const vec3 UP = vec3(0.0, 0.0, 1.0);

void main()
{ 
    // Retrieve data from gbuffer
    vec3 inFragPos = texture(gPosition, IN.TexCoords).xyz;
    vec3 inNormal = normalize(texture(gNormal, IN.TexCoords).xyz);
    vec3 inColor = texture(gColor, IN.TexCoords).xyz;
    vec3 inLight = texture(gLight, IN.TexCoords).xyz;
    vec3 inRandom = normalize(texture(gRandom, IN.TexCoords * ssaoTextureScale).xyz);

    // Compute sunlight (diffuse)
    vec3 nSunDirection = normalize((View * vec4(sunLightDirection, 0.0)).xyz);
    float sunIntensity = max(dot(nSunDirection, inNormal), 0.0);
    vec3 sunColor = sunIntensity * sunLightColor;

    // Compute ambient occlusion
    vec3 Tangent = normalize(inRandom - inNormal * dot(inRandom, inNormal));
    vec3 Bitangent = cross(inNormal, Tangent);
    mat3 TBN = mat3(Tangent, Bitangent, inNormal);  
    float ao = 0.0;
    for (int i = 0 ; i < SSAO_KERNEL_SIZE ; i++) {
        vec3 samplePos = inFragPos + (TBN * ssaoKernel[i]) * ssaoSampleRad;
        vec4 offset = vec4(samplePos, 1.0); 
        offset = Projection * offset; // project on the near clipping plane
        offset.xyz /= offset.w; // perform perspective divide
        offset.xyz = offset.xyz * 0.5 + 0.5; // transform to (0,1) range
        
        float sampleDepth = texture(gPosition, offset.xy).z;
        
        float rangeCheck = smoothstep(0.0, 1.0, ssaoSampleRad / abs(inFragPos.z - sampleDepth));
        ao += (sampleDepth >= samplePos.z + ssaoBias ? 1.0 : 0.0) * rangeCheck;    
    }
    ao = 1.0 - ao / SSAO_KERNEL_SIZE;

    // Combine ambient and sunlight
    float lightLevel = pow(inLight.r, 2);
    vec3 color = (ao * (ambientLightColor + sunColor) + lightLevel) * inColor;

    // Calculate fog
    float fogDistance = length(inFragPos);
    if (inLight.b != 1.0) {
        // This is the background
        fogDistance = fogEnd;
    }
    float fogAmount = pow(1.0 - clamp((fogEnd - fogDistance) / (fogEnd - fogStart), 0.0, 1.0), 4);
    
    // Mix fog into color
    Color = vec4(mix(color, fogColor, fogAmount), 1.0);
}
