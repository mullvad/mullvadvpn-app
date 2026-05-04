//
//  AutomaticLocationListItem.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-01.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct AutomaticLocationListItem: View {
    @State private var alert: MullvadAlert?
    private let itemFactory = ListItemFactory()

    @Binding var location: LocationNode
    var isRecent: Bool
    let onSelect: (LocationNode) -> Void

    var body: some View {
        SegmentedListItem(
            accessibilityIdentifier: isRecent ? .recentListItem(location.name) : .locationListItem(location.name),
            accessibilityLabel: location.name,
            label: {
                itemFactory.label(for: .location(node: location, context: .entry, level: 0))
            },
            segment: {
                itemFactory.segment(
                    for: .info(onSelect: {
                        alert = getAutomaticLocationInfoAlert { alert = nil }
                    })
                )
            },
            groupedContent: {},
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
                    type: .default,
                    title: "Got it!",
                    handler: completion
                )
            ]
        )
    }
}
