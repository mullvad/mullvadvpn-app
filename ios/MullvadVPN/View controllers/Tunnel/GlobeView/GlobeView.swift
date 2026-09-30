// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Logging
import RealityKit
import SwiftUI

struct GeoPoint: Hashable {
    let latitude: Double
    let longitude: Double
}

struct LocationPin {
    var id: String = UUID().uuidString
    let geoPoint: GeoPoint
    let image: Image
}

enum GlobeTarget: Equatable {
    case none
    case points([GeoPoint])
    case region(
        center: GeoPoint,
        latitudeSpan: Double = 5,
        longitudeSpan: Double = 5
    )
}

@available(iOS 18.0, *)
struct GlobeView: View {
    let target: GlobeTarget
    let locationPins: [LocationPin]
    private let logger = Logger(label: "GlobeView")
    @State private var globe: Entity? = nil
    @State private var camera: Entity? = nil
    @State private var pinContainer: Entity? = nil
    @State private var viewSize: CGSize? = nil
    var body: some View {
        RealityView { content in
            do {
                let globe = try await Entity(named: "globe")
                self.globe = globe
                let pinContainer = Entity()
                pinContainer.name = "pinContainer"
                globe.addChild(pinContainer)
                self.pinContainer = pinContainer
                content.add(globe)
                let camera = Entity()

                camera.components.set(
                    PerspectiveCameraComponent(
                        near: 0.01,
                        far: 100,
                        fieldOfViewInDegrees: 45
                    )
                )

                camera.position = [0, 0, 2]

                camera.look(
                    at: .zero,
                    from: camera.position,
                    relativeTo: nil
                )
                self.camera = camera
                content.add(camera)
                update()
            } catch {
                logger.error("Failed to load globe resource file \(error)")
            }
        } update: { content in
            update()
        }
        .sizeOfView { viewSize = $0 }
    }

    func update() {
        guard let globe,
            let viewSize,
            let camera
        else { return }
        let fitResult: GlobeFit

        switch target {
        case .none:
            fitResult = fit(
                center: .init(latitude: 57.70716, longitude: 11.96679),
                latitudeSpan: 30,
                longitudeSpan: 30,
                viewSize: viewSize
            )
        case .points(let points):
            fitResult = fit(
                points: points,
                viewSize: viewSize
            )

        case .region(
            let center,
            let latitudeSpan,
            let longitudeSpan
        ):
            fitResult = fit(
                center: center,
                latitudeSpan: latitudeSpan,
                longitudeSpan: longitudeSpan,
                viewSize: viewSize
            )
        }

        var transform = globe.transform
        transform.rotation = fitResult.rotation

        globe.move(to: transform, relativeTo: nil, duration: 2)

        camera.position = [
            0,
            0,
            fitResult.cameraDistance,
        ]

        camera.look(
            at: .zero,
            from: camera.position,
            relativeTo: nil
        )

        updatePins()
    }

    // MARK: Camera
    private let minimumCameraDistance: Float = 1.2
    private let maximumCameraDistance: Float = 5.0

    struct GlobeFit {
        let rotation: simd_quatf
        let cameraDistance: Float
        let canShowAll: Bool
    }

    private func fit(
        center: GeoPoint,
        latitudeSpan: Double,
        longitudeSpan: Double,
        viewSize: CGSize,
    ) -> GlobeFit {
        let halfLat = latitudeSpan / 2
        let halfLon = longitudeSpan / 2

        let north = min(
            90,
            center.latitude + halfLat
        )

        let south = max(
            -90,
            center.latitude - halfLat
        )

        let west =
            center.longitude - halfLon

        let east =
            center.longitude + halfLon

        let points = [
            GeoPoint(latitude: north, longitude: center.longitude),
            GeoPoint(latitude: south, longitude: center.longitude),

            GeoPoint(latitude: center.latitude, longitude: west),
            GeoPoint(latitude: center.latitude, longitude: east),

            GeoPoint(latitude: north, longitude: west),
            GeoPoint(latitude: north, longitude: east),
            GeoPoint(latitude: south, longitude: west),
            GeoPoint(latitude: south, longitude: east),
        ]

        let centerVector = unitVector(
            latitude: center.latitude,
            longitude: center.longitude
        )

        let distance = cameraDistanceThatFits(
            center: centerVector,
            points: points,
            viewSize: viewSize,
            globeRadius: 1,
            verticalFOVDegrees: 45,
            minimumDistance: minimumCameraDistance,
            maximumDistance: maximumCameraDistance
        )

        return GlobeFit(
            rotation: northUpRotation(
                latitude: center.latitude,
                longitude: center.longitude
            ),
            cameraDistance: distance,
            canShowAll: true
        )
    }

    private func fit(
        points: [GeoPoint],
        viewSize: CGSize,
    ) -> GlobeFit {
        guard let centerVector = sphericalCenter(of: points) else {
            return GlobeFit(
                rotation: simd_quatf(),
                cameraDistance: minimumCameraDistance,
                canShowAll: true
            )
        }

        let center = coordinate(
            from: centerVector
        )

        let radius = angularRadius(
            center: centerVector,
            points: points
        )

        let distance = cameraDistanceThatFits(
            center: centerVector,
            points: points,
            viewSize: viewSize,
            globeRadius: 1,
            verticalFOVDegrees: 45,
            minimumDistance: minimumCameraDistance,
            maximumDistance: maximumCameraDistance
        )

        return GlobeFit(
            rotation: northUpRotation(
                latitude: center.latitude,
                longitude: center.longitude
            ),
            cameraDistance: distance,
            canShowAll: radius <= .pi / 2
        )
    }

    private func cameraDistanceThatFits(
        center: SIMD3<Double>,
        points: [GeoPoint],
        viewSize: CGSize,
        padding: CGFloat = 96,
        globeRadius: Float,
        verticalFOVDegrees: Float,
        minimumDistance: Float,
        maximumDistance: Float
    ) -> Float {
        guard
            !points.isEmpty,
            viewSize.width > 0,
            viewSize.height > 0
        else {
            return minimumDistance
        }

        let centerCoordinate = coordinate(from: center)

        let lat = centerCoordinate.latitude * .pi / 180
        let lon = centerCoordinate.longitude * .pi / 180

        let north = simd_normalize(
            SIMD3<Double>(
                -sin(lat) * sin(lon),
                cos(lat),
                -sin(lat) * cos(lon)
            )
        )

        let east = simd_normalize(
            simd_cross(north, center)
        )

        var maxX = 0.0
        var maxY = 0.0

        for point in points {
            let p = unitVector(
                latitude: point.latitude,
                longitude: point.longitude
            )

            maxX = max(
                maxX,
                abs(simd_dot(p, east))
            )

            maxY = max(
                maxY,
                abs(simd_dot(p, north))
            )
        }

        let halfWidth =
            Float(maxX) * globeRadius

        let halfHeight =
            Float(maxY) * globeRadius

        // Full viewport dimensions.
        let width = Float(viewSize.width)
        let height = Float(viewSize.height)

        // Usable viewport after padding.
        let usableWidth = Float(
            max(1, viewSize.width - padding * 2)
        )

        let usableHeight = Float(
            max(1, viewSize.height - padding * 2)
        )

        let verticalFOV =
            verticalFOVDegrees * .pi / 180

        let aspect = width / height

        // Full frustum half-angle tangents.
        let tanVertical =
            tan(verticalFOV / 2)

        let tanHorizontal =
            tanVertical * aspect

        // Shrink each axis independently for padding.
        let usableTanVertical =
            tanVertical * (usableHeight / height)

        let usableTanHorizontal =
            tanHorizontal * (usableWidth / width)

        let distanceForHeight =
            halfHeight / usableTanVertical

        let distanceForWidth =
            halfWidth / usableTanHorizontal

        let distance =
            globeRadius
            + max(
                distanceForHeight,
                distanceForWidth
            )

        return min(
            maximumDistance,
            max(minimumDistance, distance)
        )
    }
    // MARK: Position and zoom

    // MARK: OLD zoom

    //    private func fitRegion(
    //        center: GeoPoint,
    //        latitudeSpan: Double,
    //        longitudeSpan: Double,
    //        viewSize: CGSize,
    //        padding: CGFloat = 32
    //    ) -> GlobeFit {
    //
    //        let halfLat = latitudeSpan / 2
    //        let halfLon = longitudeSpan / 2
    //
    //        let north = min(90, center.latitude + halfLat)
    //        let south = max(-90, center.latitude - halfLat)
    //
    //        let west = center.longitude - halfLon
    //        let east = center.longitude + halfLon
    //
    //        let points = [
    //            GeoPoint(latitude: north, longitude: center.longitude),
    //            GeoPoint(latitude: south, longitude: center.longitude),
    //            GeoPoint(latitude: center.latitude, longitude: west),
    //            GeoPoint(latitude: center.latitude, longitude: east),
    //
    //            // Corners help for non-square regions.
    //            GeoPoint(latitude: north, longitude: west),
    //            GeoPoint(latitude: north, longitude: east),
    //            GeoPoint(latitude: south, longitude: west),
    //            GeoPoint(latitude: south, longitude: east),
    //        ]
    //
    //        let centerVector = unitVector(
    //            latitude: center.latitude,
    //            longitude: center.longitude
    //        )
    //
    //        let scale = scaleThatFits(
    //            center: centerVector,
    //            points: points,
    //            viewSize: viewSize,
    //            padding: padding
    //        )
    //
    //        return GlobeFit(
    //            rotation: northUpRotation(
    //                latitude: center.latitude,
    //                longitude: center.longitude
    //            ),
    //            scale: scale,
    //            canShowAll: latitudeSpan <= 180 && longitudeSpan <= 180
    //        )
    //    }

    // MARK: Rotation
    private func northUpRotation(
        latitude: Double,
        longitude: Double
    ) -> simd_quatf {
        let lat = Float(latitude * .pi / 180)
        let lon = Float(longitude * .pi / 180)

        // Position of the coordinate on a unit sphere.
        let surfaceNormal = normalize(
            SIMD3<Float>(
                cos(lat) * sin(lon),
                sin(lat),
                cos(lat) * cos(lon)
            ))

        // Direction toward geographic north along the sphere.
        let north = normalize(
            SIMD3<Float>(
                -sin(lat) * sin(lon),
                cos(lat),
                -sin(lat) * cos(lon)
            ))

        // East completes the local coordinate frame.
        let east = normalize(cross(north, surfaceNormal))

        // We want:
        // surfaceNormal -> +Z (toward viewer)
        // north         -> +Y
        // east          -> +X

        let basis = simd_float3x3(
            columns: (
                east,
                north,
                surfaceNormal
            )
        )

        // basis maps local globe directions into world space,
        // so invert it to rotate the globe into our desired orientation.
        return simd_quatf(basis.transpose)
    }

    // MARK: Multiple locations

    //    private func fitGeoPoints(
    //        _ points: [GeoPoint],
    //        viewSize: CGSize,
    //        padding: CGFloat = 32
    //    ) -> GlobeFit {
    //
    //        guard
    //            let center = sphericalCenter(of: points)
    //        else {
    //            return GlobeFit(
    //                rotation: .init(angle: 0, axis: [0, 1, 0]),
    //                scale: 1,
    //                canShowAll: true
    //            )
    //        }
    //
    //        let coordinate = coordinate(from: center)
    //
    //        let radius = angularRadius(
    //            center: center,
    //            points: points
    //        )
    //
    //        let rotation = northUpRotation(
    //            latitude: coordinate.latitude,
    //            longitude: coordinate.longitude
    //        )
    //
    //        let scale = scaleThatFits(
    //            center: center,
    //            points: points,
    //            viewSize: viewSize,
    //            padding: padding
    //        )
    //
    //        return GlobeFit(
    //            rotation: rotation,
    //            scale: scale,
    //            canShowAll: radius <= .pi / 2
    //        )
    //    }
    //
    //    private func scaleThatFits(
    //        center: SIMD3<Double>,
    //        points: [GeoPoint],
    //        viewSize: CGSize,
    //        padding: CGFloat = 32
    //    ) -> Double {
    //
    //        let centerCoord = coordinate(from: center)
    //
    //        let lat = centerCoord.latitude * .pi / 180
    //        let lon = centerCoord.longitude * .pi / 180
    //
    //        let north = simd_normalize(
    //            SIMD3<Double>(
    //                -sin(lat) * sin(lon),
    //                cos(lat),
    //                -sin(lat) * cos(lon)
    //            )
    //        )
    //
    //        let east = simd_normalize(
    //            simd_cross(
    //                north,
    //                center
    //            )
    //        )
    //
    //        var maxX = 0.0
    //        var maxY = 0.0
    //
    //        for point in points {
    //            let p = unitVector(
    //                latitude: point.latitude,
    //                longitude: point.longitude
    //            )
    //
    //            maxX = max(
    //                maxX,
    //                abs(simd_dot(p, east))
    //            )
    //
    //            maxY = max(
    //                maxY,
    //                abs(simd_dot(p, north))
    //            )
    //        }
    //
    //        let usableWidth = max(
    //            1,
    //            viewSize.width - padding * 2
    //        )
    //
    //        let usableHeight = max(
    //            1,
    //            viewSize.height - padding * 2
    //        )
    //
    //        let aspect =
    //            Double(usableWidth / usableHeight)
    //
    //        /*
    //         Normalize relative to a globe whose vertical
    //         diameter at scale 1 fills the view.
    //
    //         maxY lives in -1...1.
    //         Horizontal capacity is adjusted by aspect ratio.
    //        */
    //
    //        let requiredVertical =
    //            max(
    //                maxY,
    //                maxX / aspect
    //            )
    //
    //        guard requiredVertical > 0.0001 else {
    //            return 1
    //        }
    //
    //        /*
    //         Tune maxScale for how close one point should appear.
    //
    //         Smaller values visually zoom out.
    //        */
    //        let maxScale = 1.9
    //        let minScale = 0.2
    //
    //        let scale =
    //            maxScale / max(1, requiredVertical * 2)
    //
    //        return min(
    //            maxScale,
    //            max(minScale, scale)
    //        )
    //    }

    private func unitVector(
        latitude: Double,
        longitude: Double
    ) -> SIMD3<Double> {
        let lat = latitude * .pi / 180
        let lon = longitude * .pi / 180

        return SIMD3(
            cos(lat) * sin(lon),
            sin(lat),
            cos(lat) * cos(lon)
        )
    }

    private func sphericalCenter(
        of points: [GeoPoint]
    ) -> SIMD3<Double>? {

        guard !points.isEmpty else {
            return nil
        }

        let sum = points.reduce(SIMD3<Double>.zero) {
            $0
                + unitVector(
                    latitude: $1.latitude,
                    longitude: $1.longitude
                )
        }

        let length = simd_length(sum)

        guard length > 0.000001 else {
            return nil
        }

        return sum / length
    }

    private func angularRadius(
        center: SIMD3<Double>,
        points: [GeoPoint]
    ) -> Double {

        points
            .map {
                let p = unitVector(
                    latitude: $0.latitude,
                    longitude: $0.longitude
                )

                let d = max(
                    -1.0,
                    min(1.0, simd_dot(center, p))
                )

                return acos(d)
            }
            .max() ?? 0
    }

    private func coordinate(
        from vector: SIMD3<Double>
    ) -> GeoPoint {

        let v = simd_normalize(vector)

        let latitude = asin(v.y)
        let longitude = atan2(v.x, v.z)

        return GeoPoint(
            latitude: latitude * 180 / .pi,
            longitude: longitude * 180 / .pi
        )
    }

    // MARK: Pins
    private func updatePins() {
        guard let pinContainer else { return }
        let wantedIDs = Set(
            locationPins.map { $0.id }
        )

        // Remove pins that no longer exist.
        var pinsToRemove: [Entity] = []
        for pin in pinContainer.children {
            if !wantedIDs.contains(pin.name) {
                pinsToRemove.append(pin)
            }
        }
        pinsToRemove.forEach { $0.removeFromParent() }

        // Add or update pins.
        for pin in locationPins {
            let name = pin.id

            if let pinEntity = pinContainer.findEntity(named: name) {
                updatePinPosition(
                    pinEntity,
                    for: pin.geoPoint
                )
            } else {
                Task { @MainActor in
                    guard
                        let pinEntity = try? await makePinEntity(
                            image: pin.image
                        )
                    else {
                        return
                    }

                    pinEntity.name = name
                    pinContainer.addChild(pinEntity)

                    updatePinPosition(
                        pinEntity,
                        for: pin.geoPoint
                    )
                }
            }
        }
    }

    private func updatePinPosition(
        _ pin: Entity,
        for point: GeoPoint
    ) {
        guard let globe else { return }
        let position = positionOnGlobe(
            latitude: point.latitude,
            longitude: point.longitude,
        )

        let normal = normalize(position)

        pin.position =
            position + normal * 0.01

        pin.orientation = simd_quatf(
            from: SIMD3<Float>(0, 0, 1),
            to: normal
        )
        pin.scale = 1 / globe.scale
    }

    func makePinEntity(
        image: Image,
        size: CGFloat = 512
    ) async throws -> ModelEntity {
        let renderer = ImageRenderer(
            content:
                image
                .resizable()
                .scaledToFit()
                .frame(width: size, height: size)
        )

        guard
            let uiImage = renderer.uiImage,
            let cgImage = uiImage.cgImage
        else {
            throw PinError.renderFailed
        }

        let texture = try await TextureResource(
            image: cgImage,
            options: .init(semantic: .color)
        )

        var material = UnlitMaterial()
        material.color = .init(tint: .white, texture: .init(texture))

        material.blending = .transparent(opacity: 1.0)

        return ModelEntity(
            mesh: .generatePlane(
                width: 0.04,
                height: 0.04
            ),
            materials: [material]
        )
    }

    enum PinError: Error {
        case renderFailed
    }

    private func positionOnGlobe(
        latitude: Double,
        longitude: Double,
    ) -> SIMD3<Float> {
        let globeRadius: Float = 1

        let lat = Float(latitude * .pi / 180)
        let lon = Float(longitude * .pi / 180)

        return SIMD3(
            globeRadius * cos(lat) * sin(lon),
            globeRadius * sin(lat),
            globeRadius * cos(lat) * cos(lon)
        )
    }
}

#Preview {
    @Previewable @State var span: Double = 1
    @Previewable @State var globeTarget: GlobeTarget = .region(
        center: .init(latitude: 51, longitude: 11),
        latitudeSpan: 30,
        longitudeSpan: 30
    )
    var locationPins: [LocationPin] {
        switch globeTarget {
        case .none: []
        case .points(let points):
            points.map {
                LocationPin(
                    geoPoint: $0,
                    image: Image.mullvadLocationMarkerSecure
                )
            }
        case .region(let center, _, _):
            [LocationPin(geoPoint: center, image: Image.mullvadLocationMarkerSecure)]
        }
    }
    if #available(iOS 18.0, *) {
        GlobeView(target: globeTarget, locationPins: locationPins)
        Button("Random location") {
            globeTarget =
                .region(
                    center: .init(latitude: .random(in: -90...90), longitude: .random(in: -90...90)),
                    latitudeSpan: 5,
                    longitudeSpan: 5
                )
        }
        Button("Add random location") {
            var points: [GeoPoint] =
                switch globeTarget {
                case .points(let points):
                    points
                case .region(let center, _, _):
                    [center]
                case .none: []
                }
            points.append(.init(latitude: .random(in: -90...90), longitude: .random(in: -90...90)))
            globeTarget = .points(points)
        }
        Button("Add two locations") {
            globeTarget =
                .points(
                    [
                        .init(latitude: 51, longitude: 11),
                        .init(latitude: 35, longitude: 11),
                    ]
                )
        }
        Button("Clear") {
            globeTarget = .none
        }
        Slider(value: $span, in: 0...180)
            .onChange(of: span) {
                guard case let .region(center, latitudeSpan, longitudeSpan) = globeTarget else {
                    return
                }
                globeTarget =
                    GlobeTarget
                    .region(center: center, latitudeSpan: span, longitudeSpan: span)
            }
    } else {
        // Fallback on earlier versions
    }
}
