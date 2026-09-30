#include <metal_stdlib>
using namespace metal;

struct LineVertex {
    float3 position;
};

struct LineParameters {
    float width;
    float elevation;
    uint segmentCount;
};

kernel void expandGlobePolyline(
    device const float3 *positions        [[buffer(0)]],
    device const uint *polylineIndices    [[buffer(1)]],
    device LineVertex *outVertices        [[buffer(2)]],
    device uint *outIndices               [[buffer(3)]],
    constant LineParameters &params       [[buffer(4)]],
    uint segmentID [[thread_position_in_grid]]
) {
    if (segmentID >= params.segmentCount) {
        return;
    }

    uint i0 = polylineIndices[segmentID];
    uint i1 = polylineIndices[segmentID + 1];

    float3 p0 = positions[i0];
    float3 p1 = positions[i1];

    // Slightly raise the ribbon above the globe.
    p0 *= params.elevation;
    p1 *= params.elevation;

    float3 direction = normalize(p1 - p0);

    // Globe centered at origin.
    float3 normal0 = normalize(p0);
    float3 normal1 = normalize(p1);

    // Direction across the surface, perpendicular to the line.
    float3 side0 = normalize(cross(direction, normal0));
    float3 side1 = normalize(cross(direction, normal1));

    float halfWidth = params.width * 0.5;

    float3 offset0 = side0 * halfWidth;
    float3 offset1 = side1 * halfWidth;

    uint vb = segmentID * 4;

    outVertices[vb + 0].position = p0 - offset0;
    outVertices[vb + 1].position = p0 + offset0;
    outVertices[vb + 2].position = p1 - offset1;
    outVertices[vb + 3].position = p1 + offset1;

    uint ib = segmentID * 6;

    // Two outward-facing triangles.
    outIndices[ib + 0] = vb + 0;
    outIndices[ib + 1] = vb + 1;
    outIndices[ib + 2] = vb + 2;

    outIndices[ib + 3] = vb + 1;
    outIndices[ib + 4] = vb + 3;
    outIndices[ib + 5] = vb + 2;
}
