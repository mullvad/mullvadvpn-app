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
    constexpr float IOR = 1.3;
    constexpr float DISPERSION = 0.03;
    constexpr float FRESNEL_POWER = 4.0;
    constexpr float SPEC_SHININESS = 80.0;
    constexpr float TINT_STRENGTH = 0.05;
    constexpr float LENS_DEPTH = 10.0;

    float2 center = size * 0.5;
    float2 p = position - center;
    float2 halfSize = center;
    float minDim = min(size.x, size.y);

    // 1. SDF for lens curvature (clipShape handles the visible boundary)
    float d = sdRoundedRect(p, halfSize, radius);

    // SDF gradient → direction away from nearest edge
    float eps = 0.5;
    float dx = sdRoundedRect(p + float2(eps, 0), halfSize, radius)
             - sdRoundedRect(p - float2(eps, 0), halfSize, radius);
    float dy = sdRoundedRect(p + float2(0, eps), halfSize, radius)
             - sdRoundedRect(p - float2(0, eps), halfSize, radius);
    float2 grad = float2(dx, dy) / (2.0 * eps);

    // 2. Convex lens: slope peaks near border, zero at center
    float dClamped = min(d, 0.0);
    float slope = exp(dClamped / LENS_DEPTH) / LENS_DEPTH;

    float normalStrength = slope * LENS_DEPTH * 0.6;
    float3 normal = normalize(float3(-grad * normalStrength, 1.0));

    // 3. Refraction with chromatic dispersion
    float3 eye = float3(0, 0, 1);
    float refrScale = minDim * 0.04;

    float3 refR = refract(-eye, normal, 1.0 / (IOR - DISPERSION));
    float3 refG = refract(-eye, normal, 1.0 / IOR);
    float3 refB = refract(-eye, normal, 1.0 / (IOR + DISPERSION));

    half4 sR = layer.sample(position + refR.xy * refrScale);
    half4 sG = layer.sample(position + refG.xy * refrScale);
    half4 sB = layer.sample(position + refB.xy * refrScale);

    half3 color = half3(sR.r, sG.g, sB.b);
    half alpha = max(sR.a, max(sG.a, sB.a));

    // 4. Fresnel: subtle, only at very edge (clipShape provides shape)
    float cosTheta = max(dot(eye, normal), 0.0);
    float fresnel = pow(1.0 - cosTheta, FRESNEL_POWER) * 0.08;

    // 5. Specular: gentle, from upper-left light
    float lightAngle = 0.785 + sin(time * 0.3) * 0.1;
    float3 lightDir = normalize(float3(cos(lightAngle), sin(lightAngle), 1.5));
    float3 halfVec = normalize(lightDir + eye);
    float spec = pow(max(dot(normal, halfVec), 0.0), SPEC_SHININESS) * 0.3;

    // Compose: subtle tint + gentle highlights, no rim glow
    half3 result = mix(color, half3(0.9, 0.95, 1.0), half(TINT_STRENGTH));
    result += half(fresnel + spec);
    result = clamp(result, half3(0.0), half3(1.0));

    return half4(result, alpha);
}

// --- Sparkler: streak sparks flying from orbiting points on the chip border ---
//
// Each sparking point orbits the rounded rect perimeter (golden-ratio spaced).
// Sparks are line-segment streaks, not dots. Distance-to-segment gives thin
// bright trails. Color: white-hot at source → orange → red → fade out.

// Point on rounded rect border from center, given an angle.
// Uses SDF ray marching: step outward from center until sdRoundedRect ≈ 0.
float2 pointOnRoundedRect(float angle, float2 halfSize, float r) {
    float2 dir = float2(cos(angle), sin(angle));
    // Initial guess: distance to axis-aligned bounding box
    float t = min(
        abs(dir.x) > 0.001 ? halfSize.x / abs(dir.x) : 1e6,
        abs(dir.y) > 0.001 ? halfSize.y / abs(dir.y) : 1e6
    ) * 0.9;
    // Newton-like iteration: SDF gradient along ray ≈ 1, so step by -d
    for (int i = 0; i < 8; i++) {
        float d = sdRoundedRect(dir * t, halfSize, r);
        t -= d;
    }
    return dir * t;
}

// Outward normal at a point on the rounded rect (via SDF gradient)
float2 borderNormal(float2 pt, float2 halfSize, float r) {
    float eps = 0.5;
    float dx = sdRoundedRect(pt + float2(eps, 0), halfSize, r)
             - sdRoundedRect(pt - float2(eps, 0), halfSize, r);
    float dy = sdRoundedRect(pt + float2(0, eps), halfSize, r)
             - sdRoundedRect(pt - float2(0, eps), halfSize, r);
    float2 g = float2(dx, dy);
    float gl = length(g);
    return (gl > 0.001) ? g / gl : float2(0, 1);
}

// Distance from point to a line segment (a → b)
float distToSegment(float2 p, float2 a, float2 b) {
    float2 ab = b - a;
    float2 ap = p - a;
    float t = saturate(dot(ap, ab) / max(dot(ab, ab), 0.0001));
    return length(ap - ab * t);
}

float sparkHash(float2 p) {
    return fract(sin(dot(p, float2(127.1, 311.7))) * 43758.5453);
}

[[ stitchable ]]
half4 sparkler(
    float2 position,
    half4 currentColor,
    float2 size,
    float2 chipSize,
    float time,
    float radius
) {
    float2 chipOrigin = (size - chipSize) * 0.5;
    float2 p = position - chipOrigin - chipSize * 0.5;
    float2 halfSize = chipSize * 0.5;

    constexpr int NUM_SOURCES = 5;
    constexpr int SPARKS_PER_SOURCE = 18;
    constexpr float SPARK_LIFETIME = 0.6;
    constexpr float SPARK_TRAVEL = 22.0;
    constexpr float STREAK_LENGTH = 9.0;
    constexpr float ORBIT_SPEED = 0.25;
    constexpr float PHI = 1.618033988;

    float angularVel = ORBIT_SPEED * 2.0 * M_PI_F;

    float totalBrightness = 0.0;
    float totalHeat = 0.0;

    for (int src = 0; src < NUM_SOURCES; src++) {
        float baseAngle = float(src) * PHI * 2.0 * M_PI_F;
        float currentAngle = baseAngle + time * angularVel;
        float2 sourcePos = pointOnRoundedRect(currentAngle, halfSize, radius);

        // Tight white-hot contact point on the border
        float srcDist = length(p - sourcePos);
        float srcCore = smoothstep(1.5, 0.0, srcDist); // tight bright core
        float srcGlow = smoothstep(4.0, 1.5, srcDist) * 0.3; // soft halo
        totalBrightness += srcCore + srcGlow;
        totalHeat += (srcCore + srcGlow) * 1.0;

        for (int sp = 0; sp < SPARKS_PER_SOURCE; sp++) {
            float seed = float(src * 100 + sp);

            float birthOffset = sparkHash(float2(seed, 1.0)) * SPARK_LIFETIME;
            float age = fmod(time + birthOffset, SPARK_LIFETIME);
            float life = age / SPARK_LIFETIME;

            // Birth position: where the source was when this spark was emitted
            float birthAngle = currentAngle - age * angularVel;
            float2 birthPos = pointOnRoundedRect(birthAngle, halfSize, radius);
            float2 birthNormal = borderNormal(birthPos, halfSize, radius);

            // Wide angular spread, varied speed, mostly outward
            float rSpread = sparkHash(float2(seed, 2.0)) * 2.0 - 1.0;
            float rSpeed = sparkHash(float2(seed, 3.0)) * 0.8 + 0.3;
            float rDir = sparkHash(float2(seed, 4.0)) > 0.35 ? 1.0 : -1.0;
            float rThick = sparkHash(float2(seed, 5.0)) * 0.6 + 0.6;

            float normalAngle = atan2(birthNormal.y, birthNormal.x);
            float spreadAngle = normalAngle + rSpread * 1.4; // wider spread
            float2 sparkDir = float2(cos(spreadAngle), sin(spreadAngle)) * rDir;

            // Gravity: sparks curve slightly downward as they age
            float2 gravity = float2(0.0, 3.0) * life * life;

            float travel = life * SPARK_TRAVEL * rSpeed;
            float tailLen = STREAK_LENGTH * rSpeed * (1.0 - life * 0.4);
            float2 head = birthPos + sparkDir * travel + gravity;
            float2 tail = birthPos + sparkDir * max(0.0, travel - tailLen) + gravity * 0.7;

            float dist = distToSegment(p, tail, head);

            float thickness = rThick * (1.0 - life * 0.4);
            float brightness = smoothstep(thickness + 1.2, 0.0, dist);

            float fade = 1.0 - life * life;
            brightness *= fade;

            if (brightness > 0.001) {
                totalBrightness += brightness;
                float2 ap = p - tail;
                float2 ab = head - tail;
                float along = saturate(dot(ap, ab) / max(dot(ab, ab), 0.0001));
                float heat = pow(1.0 - life, 0.4) * (0.3 + 0.7 * along);
                totalHeat += brightness * heat;
            }
        }
    }

    if (totalBrightness < 0.005) {
        return half4(0.0);
    }

    float avgHeat = totalHeat / totalBrightness;

    // Sparkler color ramp: white-hot → yellow → orange → red → out
    half3 sparkColor;
    if (avgHeat > 0.8) {
        sparkColor = mix(half3(1.0, 0.92, 0.6), half3(1.0, 1.0, 0.95), half((avgHeat - 0.8) * 5.0));
    } else if (avgHeat > 0.5) {
        sparkColor = mix(half3(1.0, 0.5, 0.05), half3(1.0, 0.92, 0.6), half((avgHeat - 0.5) / 0.3));
    } else if (avgHeat > 0.2) {
        sparkColor = mix(half3(0.8, 0.15, 0.0), half3(1.0, 0.5, 0.05), half((avgHeat - 0.2) / 0.3));
    } else {
        sparkColor = mix(half3(0.3, 0.02, 0.0), half3(0.8, 0.15, 0.0), half(avgHeat / 0.2));
    }

    half alpha = half(saturate(totalBrightness));
    return half4(sparkColor * alpha, alpha);
}
