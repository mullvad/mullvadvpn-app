//
//  FireBorder.metal
//  MullvadVPN
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

#include <metal_stdlib>
#include <SwiftUI/SwiftUI_Metal.h>
using namespace metal;

// --- Noise primitives for procedural fire ---

float hash(float2 p) {
    float3 p3 = fract(float3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

float noise(float2 p) {
    float2 i = floor(p);
    float2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = hash(i);
    float b = hash(i + float2(1.0, 0.0));
    float c = hash(i + float2(0.0, 1.0));
    float d = hash(i + float2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

float fbm(float2 p, int octaves) {
    float value = 0.0;
    float amplitude = 0.5;
    float2 shift = float2(100.0);
    for (int i = 0; i < octaves; i++) {
        value += amplitude * noise(p);
        p = p * 2.0 + shift;
        amplitude *= 0.5;
    }
    return value;
}

float sdRoundedRect(float2 p, float2 halfSize, float radius) {
    float2 q = abs(p) - halfSize + radius;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - radius;
}

// --- colorEffect: generates fire pixels around a rounded rect border ---
//
// Arguments after (position, color):
//   size      — float2(width, height) of the view
//   chipSize  — float2(width, height) of the actual chip (centered in the view)
//   time      — elapsed seconds
//   radius    — corner radius

[[ stitchable ]]
half4 fireBorder(
    float2 position,
    half4 currentColor,
    float2 size,
    float2 chipSize,
    float time,
    float radius
) {
    // The chip is centered in the larger view
    float2 chipOrigin = (size - chipSize) * 0.5;
    float2 p = position - chipOrigin - chipSize * 0.5;
    float2 halfSize = chipSize * 0.5;

    // Distance from the rounded rect border
    float d = sdRoundedRect(p, halfSize, radius);

    // Fire band: extends inward from the border to stay within chip bounds
    float outerEdge = 1.0;   // just barely outside the border
    float innerEdge = -14.0; // flames lick inward
    float bandPos = (d - innerEdge) / (outerEdge - innerEdge);

    if (bandPos < 0.0 || bandPos > 1.0) {
        return half4(0.0); // transparent outside fire region
    }

    // Invert so 0 = at border, 1 = deep inside
    bandPos = 1.0 - bandPos;

    // Angle around perimeter for racing motion
    float angle = atan2(p.y, p.x);
    float normalizedAngle = (angle + M_PI_F) / (2.0 * M_PI_F);

    // Fire turbulence: angle-based x (racing) + distance-based y (rising)
    float fireX = normalizedAngle * 14.0 - time * 3.5;
    float fireY = bandPos * 5.0 - time * 3.0;

    float turb = fbm(float2(fireX, fireY), 5);
    float turb2 = fbm(float2(fireX * 1.5 + 7.0, fireY * 1.3 + time * 1.8), 4);
    float combined = turb * 0.6 + turb2 * 0.4;

    // Intensity: strong at border, fading outward
    float falloff = 1.0 - bandPos;
    falloff = pow(falloff, 1.3);
    float intensity = combined * falloff;

    // Boost near the border for a hot edge
    float edgeBoost = smoothstep(0.25, 0.0, bandPos) * 0.5;
    intensity += edgeBoost;

    // Threshold for distinct flame tongues
    intensity = smoothstep(0.12, 0.7, intensity);

    if (intensity < 0.01) {
        return half4(0.0);
    }

    // Fire color ramp: white-hot → yellow → orange → red
    half3 fireColor;
    if (intensity > 0.85) {
        fireColor = mix(half3(1.0, 0.9, 0.3), half3(1.0, 1.0, 0.85), half(intensity - 0.85) / 0.15h);
    } else if (intensity > 0.55) {
        float t = (intensity - 0.55) / 0.3;
        fireColor = mix(half3(1.0, 0.55, 0.0), half3(1.0, 0.9, 0.3), half(t));
    } else if (intensity > 0.25) {
        float t = (intensity - 0.25) / 0.3;
        fireColor = mix(half3(0.85, 0.2, 0.0), half3(1.0, 0.55, 0.0), half(t));
    } else {
        float t = intensity / 0.25;
        fireColor = mix(half3(0.35, 0.03, 0.0), half3(0.85, 0.2, 0.0), half(t));
    }

    half fireAlpha = half(saturate(intensity * 0.95));
    // Premultiplied alpha output
    return half4(fireColor * fireAlpha, fireAlpha);
}

// --- Chromatic aberration: splits R/G/B channels apart ---
//
// layerEffect: samples the rendered chip layer with per-channel offsets.
//
// Arguments after (position, layer):
//   time      — elapsed seconds (drives oscillation)
//   intensity — base split distance in points (1-3 range)

[[ stitchable ]]
half4 chromaticAberration(
    float2 position,
    SwiftUI::Layer layer,
    float time,
    float intensity
) {
    // Oscillating split that pulses like a bass hit
    float pulse = sin(time * 4.0) * 0.5 + 0.5; // 0..1
    float split = intensity * (0.6 + pulse * 0.4);

    // RGB channels offset in different directions (120° apart)
    float angle = time * 0.7; // slowly rotate the split direction
    float2 rDir = float2(cos(angle), sin(angle));
    float2 gDir = float2(cos(angle + 2.094), sin(angle + 2.094)); // +120°
    float2 bDir = float2(cos(angle + 4.189), sin(angle + 4.189)); // +240°

    half4 rSample = layer.sample(position + rDir * split);
    half4 gSample = layer.sample(position + gDir * split);
    half4 bSample = layer.sample(position + bDir * split);

    half4 result;
    result.r = rSample.r;
    result.g = gSample.g;
    result.b = bSample.b;
    result.a = max(rSample.a, max(gSample.a, bSample.a));

    return result;
}

// --- Liquid glass: Apple-style SDF lens refraction + Fresnel + Blinn-Phong ---
//
// Modeled after Apple's Liquid Glass design language (iOS 26):
//  1. Rounded rect SDF → surface normals (convex lens curvature)
//  2. Snell's law refraction offsets UV sampling (IOR ~1.3)
//  3. Chromatic dispersion: per-channel IOR shift at edges
//  4. Fresnel edge reflection (Schlick's approximation)
//  5. Blinn-Phong specular from a gently drifting light direction
//  6. Rim glow + corner boost at high-curvature SDF regions
//
// layerEffect arguments after (position, layer):
//   size      — float2(width, height) of the view
//   time      — elapsed seconds (drives subtle light drift)
//   radius    — corner radius of the rounded rect

[[ stitchable ]]
half4 liquidGlass(
    float2 position,
    SwiftUI::Layer layer,
    float2 size,
    float time,
    float radius
) {
    constexpr float IOR = 1.31;
    constexpr float DEPTH = 0.15;
    constexpr float DISPERSION = 0.025;
    constexpr float FRESNEL_POWER = 3.0;
    constexpr float SPEC_SHININESS = 64.0;
    constexpr float LIGHT_INTENSITY = 0.55;
    constexpr float RIM_INTENSITY = 0.08;
    constexpr float TINT_STRENGTH = 0.06;

    float2 center = size * 0.5;
    float2 p = position - center;
    // Inset by 1px so the rounded rect boundary falls inside the view
    float2 halfSize = center - 1.0;
    float minDim = min(size.x, size.y);

    // 1. SDF and surface normal via central differences
    float d = sdRoundedRect(p, halfSize, radius);

    // Discard pixels outside the rounded rect (show as transparent)
    if (d > 0.0) {
        return layer.sample(position);
    }
    float eps = 0.5;
    float dx = sdRoundedRect(p + float2(eps, 0), halfSize, radius)
             - sdRoundedRect(p - float2(eps, 0), halfSize, radius);
    float dy = sdRoundedRect(p + float2(0, eps), halfSize, radius)
             - sdRoundedRect(p - float2(0, eps), halfSize, radius);
    float2 grad = float2(dx, dy) / (2.0 * eps);

    // Convex lens curvature: strongest at center, fades at border
    float inside = smoothstep(0.0, -minDim * DEPTH, d);
    float curvature = inside * DEPTH * minDim;
    float3 normal = normalize(float3(-grad * curvature * 0.02, 1.0));

    // 2–3. Refraction with chromatic dispersion (per-channel IOR)
    float3 eye = float3(0, 0, 1);
    float scale = minDim * 0.03;

    float3 refR = refract(-eye, normal, 1.0 / (IOR - DISPERSION));
    float3 refG = refract(-eye, normal, 1.0 / IOR);
    float3 refB = refract(-eye, normal, 1.0 / (IOR + DISPERSION));

    half4 sR = layer.sample(position + refR.xy * scale);
    half4 sG = layer.sample(position + refG.xy * scale);
    half4 sB = layer.sample(position + refB.xy * scale);

    half3 color = half3(sR.r, sG.g, sB.b);
    half alpha = max(sR.a, max(sG.a, sB.a));

    // 4. Fresnel reflection (Schlick)
    float cosTheta = max(dot(eye, normal), 0.0);
    float fresnel = pow(1.0 - cosTheta, FRESNEL_POWER) * 0.15;

    // 5. Blinn-Phong specular with gently drifting light
    float lightAngle = 0.785 + sin(time * 0.3) * 0.15;
    float3 lightDir = normalize(float3(cos(lightAngle), sin(lightAngle), 1.2));
    float3 halfVec = normalize(lightDir + eye);
    float spec = pow(max(dot(normal, halfVec), 0.0), SPEC_SHININESS) * LIGHT_INTENSITY;

    // 6. Rim glow + corner boost
    float edgeFade = smoothstep(0.0, -4.0, d);
    float rim = (1.0 - edgeFade) * RIM_INTENSITY;
    float cornerMask = smoothstep(0.8, 1.4, length(grad));
    rim += cornerMask * 0.04;

    // Compose
    half3 result = mix(color, half3(0.88, 0.94, 1.0), half(TINT_STRENGTH));
    result += half(fresnel + spec + rim);
    result = clamp(result, half3(0.0), half3(1.0));

    return half4(result, alpha);
}
