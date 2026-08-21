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

struct RecentLocationsListView<ContextMenu>: View where ContextMenu: View {
    @Binding var locations: [LocationNode]
    let multihopContext: MultihopContext
    let onSelectLocation: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu

    var body: some View {
        ForEach($locations, id: \.self) { location in
            RecentLocationListItem(
                location: location,
                multihopContext: multihopContext,
                onSelect: onSelectLocation,
                contextMenu: { location in contextMenu(location) },
            )
        }
    }
}
