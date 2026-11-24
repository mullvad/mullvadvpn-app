// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI

@Observable
class GlobeViewModel {
    var globeTarget = GlobeTarget.none
    var locationPins: [LocationPin] = []
    var isShowingSpinner = false

    func removeLocationMarker() {
        locationPins = []
    }

    func addLocationMarker(coordinate: CLLocationCoordinate2D) {
        locationPins
            .append(
                .init(
                    geoPoint: .init(
                        latitude: coordinate.latitude,
                        longitude: coordinate
                            .longitude),
                    image: Image.mullvadLocationMarkerSecure
                )
            )
    }

    func showLocations(_ locations: [CLLocationCoordinate2D]) {
        if locations.isEmpty {
            globeTarget = .none
        } else if locations.count == 1,
            let center = locations.first
        {
            globeTarget =
                .region(
                    center: GeoPoint(
                        latitude: center.latitude,
                        longitude: center
                            .longitude)
                )
        } else {
            globeTarget =
                .points(
                    locations
                        .map {
                            GeoPoint(
                                latitude: $0.latitude,
                                longitude: $0.longitude
                            )
                        })
        }
    }

    func setCenter(_ center: CLLocationCoordinate2D?) {
        var locations = [CLLocationCoordinate2D]()
        if let center {
            locations.append(center)
        }
        showLocations(locations)
    }
}
