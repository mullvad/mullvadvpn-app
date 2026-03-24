//
//  RecentLocationListItem.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct RecentLocationListItem<ContextMenu>: View where ContextMenu: View {
    @State private var alert: MullvadAlert?
    private let itemFactory = ListItemFactory()

    @Binding var location: LocationNode
    let onSelect: (LocationNode) -> Void
    let contextMenu: (LocationNode) -> ContextMenu

    var body: some View {
        let isAutomaticLocation = location is AutomaticLocationNode

        SegmentedListItem(
            accessibilityIdentifier: .recentListItem(location.name),
            accessibilityLabel: location.name,
            label: {
                itemFactory.label(for: .recent(node: location))
            },
            segment: {
                if isAutomaticLocation {
                    itemFactory.segment(
                        for: .info(onSelect: {
                            alert = getAutomaticLocationInfoAlert { alert = nil }
                        })
                    )
                }
            },
            groupedContent: {},
            onSelect: {
                onSelect(location)
            }
        )
        .contextMenu {
            isAutomaticLocation ? nil : contextMenu(location)
        }
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
