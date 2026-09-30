// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MapKit
import SwiftUI

@Observable
class GlobeViewModel {
    var globeTarget = GlobeTarget.none
    var locationPins: [LocationPin] = []

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

    func setCenter(_ center: CLLocationCoordinate2D?, animated: Bool) {
        guard let center else {
            globeTarget = .none
            return
        }
        globeTarget =
            .region(
                center: .init(
                    latitude: center.latitude,
                    longitude: center.longitude
                )
            )
    }
}

struct GlobeViewContainer: View {
    var viewModel: GlobeViewModel
    var body: some View {
        if #available(iOS 18.0, *) {
            GlobeView(target: viewModel.globeTarget, locationPins: viewModel.locationPins)
                .ignoresSafeArea()
        } else {
            // Fallback on earlier versions
        }
    }
}
