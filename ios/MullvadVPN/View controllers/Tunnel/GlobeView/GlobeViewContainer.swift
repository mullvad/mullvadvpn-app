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

struct GlobeViewContainer: View {
    @State var viewModel: GlobeViewModel
    var offset: CGFloat = 0  // 0 none, -1 top, 1 bottom
    var body: some View {
        if #available(iOS 18.0, *) {
            GeometryReader { geo in
                let h = geo.size.height
                let vh = h * (1 + abs(offset))
                ZStack {
                    mullvadLoadingSpinner(isPresented: viewModel.isShowingSpinner)
//                    GlobeView(target: viewModel.globeTarget, locationPins: viewModel.locationPins)
//                        .ignoresSafeArea()
//                        .frame(width: geo.size.width, height: vh)
                }
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
                viewModel.setCenter(center)
                viewModel.addLocationMarker(coordinate: center)
            }
            .padding()
        }
    }
}
