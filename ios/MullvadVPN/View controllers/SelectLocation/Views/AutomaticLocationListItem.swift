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

struct AutomaticLocationListItem: View {
    @State private var alert: MullvadAlert?
    private let itemFactory = SegmentedListItemFactory()

    @Binding var location: LocationNode
    var isRecent: Bool
    let onSelect: (LocationNode) -> Void

    var body: some View {
        Color.clear.frame(height: 4).accessibilityHidden(true)

        SegmentedListItem(
            accessibilityIdentifier: isRecent ? .recentListItem(location.name) : .locationListItem(location.name),
            accessibilityLabel: location.name,
            leading: {
                itemFactory.leading(for: .location(node: location, context: .entry, level: 0))
            },
            segment: {
                itemFactory.segment(
                    for: .info(onSelect: {
                        alert = getAutomaticLocationInfoAlert { alert = nil }
                    })
                )
            },
            onSelect: {
                onSelect(location)
            }
        )
        .id(location.id)  // to be able to scroll to this item programmatically
        .mullvadAlert(item: $alert)
    }

    func getAutomaticLocationInfoAlert(completion: @escaping () -> Void) -> MullvadAlert {
        let message = [
            (NSLocalizedString(
                "When the “Automatic” location is selected, the app automatically picks a random entry server, "
                    + "prioritizing those closer to the exit location for better performance", comment: "")),
            (NSLocalizedString(
                "Attention: With the “Automatic” location, any enabled filters are ignored for the entry "
                    + "server.", comment: "")),
        ].joinedParagraphs()

        return MullvadAlert(
            type: .info,
            messages: [LocalizedStringKey(message)],
            actions: [
                MullvadAlert.Action(
                    type: .primary,
                    title: "Got it!",
                    handler: completion
                )
            ]
        )
    }
}
