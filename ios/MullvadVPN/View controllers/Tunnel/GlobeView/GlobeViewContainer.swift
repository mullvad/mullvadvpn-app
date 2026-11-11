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
    @State var viewModel: GlobeViewModel
    var offset: CGFloat = 0  // 0 none, -1 top, 1 bottom
    var body: some View {
        if #available(iOS 18.0, *) {
            GeometryReader { geo in
                let h = geo.size.height
                let vh = h * (1 + abs(offset))
                GlobeView(target: viewModel.globeTarget, locationPins: viewModel.locationPins)
                    .ignoresSafeArea()
                    .frame(width: geo.size.width, height: vh)
                    .offset(y: min(offset, 0) * h)
                    .clipped()
                    .background(Color.mullvadDarkBackground.ignoresSafeArea())
            }
        } else {
            // Fallback on earlier versions
        }
    }
}

#Preview {
    @Previewable @State var viewModel = GlobeViewModel()
    ZStack {
        GlobeViewContainer(viewModel: viewModel, offset: -1 / 2)
        VStack {
            Spacer()
            MullvadButton(text: "Random position", style: .primary) {
                viewModel.removeLocationMarker()
                let center = CLLocationCoordinate2D(latitude: .random(in: -90...90), longitude: .random(in: -90...90))
                viewModel.setCenter(center, animated: true)
                viewModel.addLocationMarker(coordinate: center)
            }
            .padding()
        }
    }
}
