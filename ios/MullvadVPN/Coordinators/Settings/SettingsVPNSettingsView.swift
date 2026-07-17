//
//  SettingsVPNSettingsView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2026-07-17.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

private struct GlobalVPNSetting: Identifiable {
    let id: String
    let label: String
    let accessibilityIdentifier: AccessibilityIdentifier
    let customView: AnyView?
}

struct SettingsVPNSettingsView: View {
    private let itemFactory = SegmentedListItemFactory()

    private let DNSandIPSettings: [GlobalVPNSetting] = [
        .init(
            id: NSLocalizedString("DNS settings", comment: ""),
            label: NSLocalizedString("DNS settings", comment: ""),
            accessibilityIdentifier: .dnsSettings, customView: nil),
        .init(
            id: NSLocalizedString("Server IP override", comment: ""),
            label: NSLocalizedString("Server IP override", comment: ""),
            accessibilityIdentifier: .dnsSettings, customView: nil),
    ]

    var body: some View {
        SettingsInfoContainerView {
            VStack(alignment: .leading, spacing: 1) {
                SegmentedListItem(
                    userInteraction: .enabled,
                    accessibilityIdentifier: .vpnSettingsTableView,
                    leading: {
                        SegmentedListItem(
                            isLastInList: false,
                            userInteraction: .enabled,
                            accessibilityIdentifier: DNSandIPSettings.first?.accessibilityIdentifier,
                            leading: {
                                itemFactory.leading(
                                    for: .generic(title: DNSandIPSettings.first!.label, level: 0, isSelected: false))
                            },
                            trailing: {
                                itemFactory.trailing(for: .drillDown(title: ""))
                            }
                        )
                    },
                    groupedContent: {
                        SegmentedListItem(
                            userInteraction: .enabled,
                            accessibilityIdentifier: DNSandIPSettings.last?.accessibilityIdentifier,
                            leading: {
                                itemFactory.leading(
                                    for: .generic(title: DNSandIPSettings.last!.label, level: 0, isSelected: false))
                            },
                            trailing: {
                                itemFactory.trailing(for: .drillDown(title: ""))
                            }
                        )
                    }
                )
                .padding(.leading, UIMetrics.contentInsets.left)
                .padding(.trailing, UIMetrics.contentInsets.right)
            }
        }
    }
}

#Preview {
    SettingsVPNSettingsView()
}
