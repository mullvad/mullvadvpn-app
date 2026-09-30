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

let oceanMaterial = UnlitMaterial(
    color: .init(red: 0.10, green: 0.18, blue: 0.27, alpha: 1),
    applyPostProcessToneMap: true
)

let oceanEntity = ModelEntity(
    mesh: oceanMesh,
    materials: [oceanMaterial]
)

let lineMeshGenerator = try! LineMeshGenerator()
let lowLevelMesh = try! lineMeshGenerator.makeMesh(
    positions: globeData.landVertices,
    polylineIndices: globeData.contourIndices,
    width: 0.001
)

let meshResource = try! MeshResource(
    from: lowLevelMesh
)

let contourEntity = ModelEntity(
    mesh: meshResource,
    materials: [oceanMaterial]
)

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

struct LineVertex {
    var position: SIMD3<Float>
}

struct LineParameters {
    var width: Float
    var elevation: Float
    var segmentCount: UInt32
}

@MainActor
final class LineMeshGenerator {

    let device: MTLDevice
    let commandQueue: MTLCommandQueue
    let pipeline: MTLComputePipelineState

    init() throws {
        let device = MTLCreateSystemDefaultDevice()!
        let source = try String(
            contentsOfFile: "LineMeshShader.metal",
            encoding: .utf8
        )
        let library = try device.makeLibrary(
            source: source,
            options: nil
        )
        let function = library.makeFunction(name: "expandGlobePolyline")!
        let commandQueue = device.makeCommandQueue()!

        self.device = device
        self.commandQueue = commandQueue
        self.pipeline = try device.makeComputePipelineState(
            function: function
        )
    }

    func makeMesh(
        positions: [SIMD3<Float>],
        polylineIndices: [UInt32],
        width: Float,
        elevation: Float = 1.000
    ) throws -> LowLevelMesh {

        guard polylineIndices.count >= 2 else {
            fatalError("Not enough indices")
        }

        let segmentCount = polylineIndices.count - 1

        let vertexCount = segmentCount * 4
        let triangleIndexCount = segmentCount * 6

        let attributes: [LowLevelMesh.Attribute] = [
            .init(
                semantic: .position,
                format: .float3,
                offset: MemoryLayout<LineVertex>.offset(
                    of: \.position
                )!
            )
        ]

        let layouts: [LowLevelMesh.Layout] = [
            .init(
                bufferIndex: 0,
                bufferOffset: 0,
                bufferStride: MemoryLayout<LineVertex>.stride
            )
        ]

        let descriptor = LowLevelMesh.Descriptor(
            vertexCapacity: vertexCount,
            vertexAttributes: attributes,
            vertexLayouts: layouts,
            indexCapacity: triangleIndexCount,
            indexType: .uint32
        )

        let mesh = try LowLevelMesh(
            descriptor: descriptor
        )

        try populateMesh(
            mesh,
            positions: positions,
            polylineIndices: polylineIndices,
            width: width,
            elevation: elevation
        )

        return mesh
    }

    private func populateMesh(
        _ mesh: LowLevelMesh,
        positions: [SIMD3<Float>],
        polylineIndices: [UInt32],
        width: Float,
        elevation: Float
    ) throws {

        let segmentCount = polylineIndices.count - 1

        let sourcePositions = device.makeBuffer(
            bytes: positions,
            length:
                positions.count * MemoryLayout<SIMD3<Float>>.stride,
            options: .storageModeShared
        )!

        let sourceIndices = device.makeBuffer(
            bytes: polylineIndices,
            length:
                polylineIndices.count * MemoryLayout<UInt32>.stride,
            options: .storageModeShared
        )!

        let commandBuffer =
            commandQueue.makeCommandBuffer()!

        // These are the RealityKit-owned buffers that
        // our Metal shader will write into.
        let destinationVertices = mesh.replace(
            bufferIndex: 0,
            using: commandBuffer
        )

        let destinationIndices = mesh.replaceIndices(
            using: commandBuffer
        )

        let encoder =
            commandBuffer.makeComputeCommandEncoder()!

        encoder.setComputePipelineState(pipeline)

        // [[buffer(0)]]
        encoder.setBuffer(
            sourcePositions,
            offset: 0,
            index: 0
        )

        // [[buffer(1)]]
        encoder.setBuffer(
            sourceIndices,
            offset: 0,
            index: 1
        )

        // [[buffer(2)]]
        encoder.setBuffer(
            destinationVertices,
            offset: 0,
            index: 2
        )

        // [[buffer(3)]]
        encoder.setBuffer(
            destinationIndices,
            offset: 0,
            index: 3
        )

        var parameters = LineParameters(
            width: width,
            elevation: elevation,
            segmentCount: UInt32(segmentCount)
        )

        // [[buffer(4)]]
        encoder.setBytes(
            &parameters,
            length: MemoryLayout<LineParameters>.stride,
            index: 4
        )

        let threadsPerGroup = min(
            pipeline.maxTotalThreadsPerThreadgroup,
            pipeline.threadExecutionWidth
        )

        let threadgroups = (segmentCount + threadsPerGroup - 1) / threadsPerGroup

        encoder.dispatchThreadgroups(
            MTLSize(width: threadgroups, height: 1, depth: 1),
            threadsPerThreadgroup: MTLSize(
                width: threadsPerGroup,
                height: 1,
                depth: 1
            )
        )

        encoder.endEncoding()

        commandBuffer.commit()

        let radius: Float = 1

        mesh.parts.replaceAll([
            LowLevelMesh.Part(
                indexCount: segmentCount * 6,
                topology: .triangle,
                bounds: BoundingBox(
                    min: [-radius, -radius, -radius],
                    max: [radius, radius, radius]
                )
            )
        ])
    }
}
