//
//  Midpoint.swift
//  RelaySelector
//
//  Created by Marco Nikic on 2023-07-11.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import CoreLocation
import Foundation
import MullvadTypes

public enum Midpoint {
    /// Computes the approximate midpoint of a set of locations.
    ///
    /// This works by calculating the mean Cartesian coordinates, and converting them
    /// back to spherical coordinates. This is approximate, because the semi-minor (polar)
    /// axis is assumed to equal the semi-major (equatorial) axis.
    ///
    /// https://en.wikipedia.org/wiki/Spherical_coordinate_system#Cartesian_coordinates
    static func location(in coordinates: [CLLocationCoordinate2D]) -> CLLocationCoordinate2D {
        var x = 0.0, y = 0.0, z = 0.0
        var count = 0

        coordinates.forEach { coordinate in
            let cos_lat = cos(coordinate.latitude.toRadians)
            let sin_lat = sin(coordinate.latitude.toRadians)
            let cos_lon = cos(coordinate.longitude.toRadians)
            let sin_lon = sin(coordinate.longitude.toRadians)

            x += cos_lat * cos_lon
            y += cos_lat * sin_lon
            z += sin_lat

            count += 1
        }

        let inv_total_weight = 1.0 / Double(count)
        x *= inv_total_weight
        y *= inv_total_weight
        z *= inv_total_weight

        let longitude = atan2(y, x)
        let hypotenuse = sqrt(x * x + y * y)
        let latitude = atan2(z, hypotenuse)

        return CLLocationCoordinate2D(latitude: latitude.toDegrees, longitude: longitude.toDegrees)
    }
}
