#version 330 core

out vec3 fragColor;

in vec3 vertNormal;

const vec3 UP = vec3(0.0, 0.0, -1.0);

uniform sampler2D ditherTexture;

uniform vec3 fogColor;
uniform vec3 skyColor;
uniform vec3 sunLightDirection;
uniform vec3 sunColor;

uniform mat4 View;
uniform mat4 Projection;

void main()
{
    // Calculate small "dither" to hide color banding
    vec3 dither = vec3(texture2D(ditherTexture, gl_FragCoord.xy / 8.0).r / 32.0 - (1.0 / 128.0));
    // Color sky based on distance from horizon
    float angle = clamp(dot(vertNormal, UP), 0.0, 1.0);
    // vec3 groundColor = fogColor * 0.75;
    // fragColor = mix(groundColor, fogColor, smoothstep(0.00, 0.05, angle));
    fragColor = mix(fogColor, skyColor, smoothstep(0.1, 0.5, angle)) + dither;
    
    // Calculate sun color and its glow
    float sun = clamp(1.0 - distance(vertNormal, sunLightDirection), 0.0, 1.0);
    float glow = sun;
    sun = pow(sun,100.0);
    sun *= 100.0;
    sun = clamp(sun,0.0,1.0);
    glow = pow(glow,6.0) * 1.0;
    glow = pow(glow,clamp(1.0 - sunLightDirection.z, 0.0, 1.0));
    glow = clamp(glow,0.0,1.0);
    
    fragColor = fragColor + clamp(sun + glow, 0.0, 1.0) * sunColor;
}