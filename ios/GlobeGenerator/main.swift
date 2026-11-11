// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Metal
import RealityKit

let globeData = GlobeData()
var landDescriptor = MeshDescriptor(name: "LandTriangle")
landDescriptor.positions = MeshBuffers.Positions(globeData.landVertices)
landDescriptor.primitives = .triangles(globeData.landIndices)

var oceanDescriptor = MeshDescriptor(name: "OceanTriangle")
oceanDescriptor.positions =
    MeshBuffers
    .Positions(globeData.oceanVertices)
oceanDescriptor.primitives = .triangles(globeData.oceanIndices)

var contoursDescriptor = MeshDescriptor(name: "ContoursTriangle")
contoursDescriptor.positions =
    MeshBuffers
    .Positions(globeData.landVertices)
contoursDescriptor.primitives = .triangles(globeData.contourIndices)

let landMesh = try! MeshResource.generate(from: [landDescriptor])

let landMaterial = UnlitMaterial(
    color: .init(red: 0.16, green: 0.3, blue: 0.45, alpha: 1),
    applyPostProcessToneMap: true
)

let landEntity = ModelEntity(
    mesh: landMesh,
    materials: [landMaterial]
)

let oceanMesh = try! MeshResource.generate(from: [oceanDescriptor])

var oceanMaterial = UnlitMaterial(
    color: .init(red: 0.10, green: 0.18, blue: 0.27, alpha: 1),
    applyPostProcessToneMap: true
)

let oceanEntity = ModelEntity(
    mesh: oceanMesh,
    materials: [oceanMaterial]
)

// Usage
let mesh = try makeSphereStripMesh(positions: globeData.landVertices, indices: globeData.contourIndices, halfWidth: 0.001)
oceanMaterial.faceCulling = .none                               // winding varies with segment direction
let contourEntity = ModelEntity(mesh: mesh, materials: [oceanMaterial])

let globeEntity = Entity()
globeEntity.addChild(landEntity)
globeEntity.addChild(oceanEntity)
globeEntity.addChild(contourEntity)
globeEntity.name = "globe"

let url = URL(filePath: "../Assets/globe.reality")!
try! await globeEntity.write(to: url)

struct GlobeData {
    let oceanVertices: [SIMD3<Float>]

    let landVertices: [SIMD3<Float>]

    let landIndices: [UInt32]

    let oceanIndices: [UInt32]

    let contourIndices: [UInt32]

    init() {
        // Load land vertices
        let baseURL = URL(filePath: "../../dist-assets/geo")!
        let landVerticesData = try! Data(contentsOf: baseURL.appending(path: "land_positions.gl"))
        landVertices = landVerticesData.withUnsafeBytes { rawBuffer in
            let floats = rawBuffer.bindMemory(to: Float.self)

            return stride(from: 0, to: floats.count, by: 3).map { i in
                SIMD3(
                    floats[i],
                    floats[i + 1],
                    floats[i + 2]
                )
            }
        }

        // Load ocean vertices
        let oceanVertexBuffer = try! Data(contentsOf: baseURL.appending(path: "ocean_positions.gl"))

        oceanVertices = oceanVertexBuffer.withUnsafeBytes { rawBuffer in
            let floats = rawBuffer.bindMemory(to: Float.self)

            return stride(from: 0, to: floats.count, by: 3).map { i in
                SIMD3(
                    floats[i],
                    floats[i + 1],
                    floats[i + 2]
                )
            }
        }

        // Load land triangle indices
        let landIndices = try! Data(contentsOf: baseURL.appending(path: "land_triangle_indices.gl"))
        self.landIndices = landIndices.withUnsafeBytes { rawBuffer in
            Array(rawBuffer.bindMemory(to: UInt32.self))
        }

        // Load land contour indices
        let contourIndexData = try! Data(contentsOf: baseURL.appending(path: "land_contour_indices.gl"))
        contourIndices = contourIndexData.withUnsafeBytes { rawBuffer in
            Array(rawBuffer.bindMemory(to: UInt32.self))
        }

        // Load ocean triangle indices
        let oceanIndexData = try! Data(contentsOf: baseURL.appending(path: "ocean_indices.gl"))
        oceanIndices = oceanIndexData.withUnsafeBytes { rawBuffer in
            Array(rawBuffer.bindMemory(to: UInt32.self))
        }
    }
}

func makeSphereStripMesh(positions: [SIMD3<Float>],  // on the sphere, centered at origin
                         indices: [UInt32],          // line strip: i0, i1, i2, ...
                         halfWidth: Float,
                         lift: Float = 1.0001) throws -> MeshResource {
    let count = indices.count
    precondition(count >= 2)

    let pts = indices.map { positions[Int($0)] }

    // Tangent-plane direction from p towards q, at point p
    func dir(from p: SIMD3<Float>, to q: SIMD3<Float>) -> SIMD3<Float> {
        let n = simd_normalize(p)
        var d = q - p
        d -= n * simd_dot(n, d)                       // project into tangent plane
        let l = simd_length(d)
        return l > 1e-9 ? d / l : .zero
    }

    var verts: [SIMD3<Float>] = []
    var normals: [SIMD3<Float>] = []
    verts.reserveCapacity(count * 2)
    normals.reserveCapacity(count * 2)

    for k in 0..<count {
        let p = pts[k]
        let n = simd_normalize(p)

        let dPrev = k > 0         ? dir(from: pts[k - 1], to: p) : .zero
        let dNext = k < count - 1 ? dir(from: p, to: pts[k + 1]) : .zero

        var t = dPrev + dNext
        if simd_length(t) < 1e-6 { t = (dNext != .zero) ? dNext : dPrev }  // endpoints / U-turns
        t = simd_normalize(t)

        let side = simd_normalize(simd_cross(n, t))

        // Miter: widen so the ribbon keeps its width across the corner
        var w = halfWidth
        let segDir = (dNext != .zero) ? dNext : dPrev
        let segSide = simd_cross(n, segDir)
        let c = simd_dot(side, segSide)
        w = halfWidth / max(c, 0.5)                   // clamp to avoid spikes at sharp angles

        let center = p * lift
        verts.append(center - side * w); normals.append(n)
        verts.append(center + side * w); normals.append(n)
    }

    var tris: [UInt32] = []
    tris.reserveCapacity((count - 1) * 6)
    for k in 0..<(count - 1) {
        let a = UInt32(k * 2), b = UInt32((k + 1) * 2)
        tris += [a, b, a + 1,   a + 1, b, b + 1]      // CCW seen from outside the sphere
    }

    var desc = MeshDescriptor(name: "sphereStrip")
    desc.positions = MeshBuffer(verts)
     desc.normals = MeshBuffer(normals)
    desc.primitives = .triangles(tris)
    return try MeshResource.generate(from: [desc])
}
