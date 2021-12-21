#version 330 core

out vec3 fragColor;

in vec3 vertNormal;

const vec3 UP = vec3(0.0, 0.0, -1.0);

uniform vec3 fogColor;
uniform vec3 skyColor;
uniform vec3 sunLightDirection;
uniform vec3 sunColor;

uniform mat4 View;
uniform mat4 Projection;

void main()
{
    // Color sky based on distance from horizon
    float angle = clamp(dot(vertNormal, UP), 0.0, 1.0);
    fragColor = mix(fogColor, skyColor, smoothstep(0.05, 0.25, angle));
    
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