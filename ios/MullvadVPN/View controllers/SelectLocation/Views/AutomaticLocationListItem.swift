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
                "Picks a suitable location based on your exit location, this is based on a number "
                    + "of different factors such as distance, provider, and server load.", comment: "")),
            (NSLocalizedString(
                "Attention: This will ignore filter settings for the server that is being "
                    + "used as an entry point", comment: "")),
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
