//
//  Haversine.swift
//  RelaySelector
//
//  Created by Marco Nikic on 2023-06-29.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public enum Haversine {
    /// Approximation of the radius of the average circumference,
    /// where the boundaries are the meridian (6367.45 km) and the equator (6378.14 km).
    static let earthRadiusInKm = 6372.8

    /// Implemented as per https://rosettacode.org/wiki/Haversine_formula#Swift
    /// Computes the great circle distance between two points on a sphere.
    ///
    /// The inputs are converted to radians, and the output is in kilometers.
    /// - Parameters:
    ///   - lat1: The first point's latitude
    ///   - lon1: The first point's longitude
    ///   - lat2: The second point's latitude
    ///   - lon2: The second point's longitude
    /// - Returns: The haversine distance between the two points.
    static func distance(
        _ latitude1: Double,
        _ longitude1: Double,
        _ latitude2: Double,
        _ longitude2: Double
    ) -> Double {
        let dLat = latitude1.toRadians - latitude2.toRadians
        let dLon = longitude1.toRadians - longitude2.toRadians

        let haversine = sin(dLat / 2).squared + sin(dLon / 2)
            .squared * cos(latitude1.toRadians) * cos(latitude2.toRadians)
        let c = 2 * asin(sqrt(haversine))

        return Self.earthRadiusInKm * c
    }
}

extension Double {
    var toRadians: Double { self * Double.pi / 180.0 }
    var toDegrees: Double { self * 180.0 / Double.pi }
    var squared: Double { pow(self, 2.0) }
}
