// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes
import SwiftUI

struct CustomListLocationSelectionView: View {
    @Bindable var availableLocations: CustomListSelectableLocationNode

    var body: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                LocationsListView(
                    locations: $availableLocations.children,
                    multihopContext: .entry,
                    onSelectLocation: {
                        ($0 as? CustomListSelectableLocationNode)?.isCustomListSelected.toggle()
                    },
                    contextMenu: { _ in EmptyView() }
                )
            }
            .padding()
        }
        .background(Color.mullvadBackground)
        .accessibilityIdentifier(.editCustomListEditLocationsView)
    }
}

#Preview {
    @Previewable @State var availableLocations = MockSelectLocationViewModel().exitContext.locations
    CustomListLocationSelectionView(
        availableLocations: CustomListSelectableLocationNode(
            fromLocationNode: LocationNode(name: "", code: "", children: availableLocations))
    )
}
