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
