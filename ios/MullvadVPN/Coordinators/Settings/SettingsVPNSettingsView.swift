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

struct SettingsVPNSettingsView<ViewModel>: View where ViewModel: ObservableVPNSettings {
    private let itemFactory = SegmentedListItemFactory()

    @ObservedObject var viewModel: ViewModel

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

    private let censorshipSettings: [GlobalVPNSetting] = [
        .init(
            id: NSLocalizedString("Anti-censorship", comment: ""),
            label: NSLocalizedString("Anti-censorship", comment: ""),
            accessibilityIdentifier: .dnsSettings, customView: nil),
        .init(
            id: NSLocalizedString("Quantum-resistant tunnel", comment: ""),
            label: NSLocalizedString("Quantum-resistant tunnel", comment: ""),
            accessibilityIdentifier: .dnsSettings, customView: nil),
    ]

    var body: some View {
        SettingsInfoContainerView {
            DNSandIPSettingsView()
            antiCensorshipView()
        }
    }

    var isQuantumResistanceEnabled: Binding<Bool> {
        Binding<Bool>(
            get: {
                viewModel.quantumResistance.isEnabled
            },
            set: { enabled in
                viewModel.quantumResistance = enabled ? .on : .off
            }
        )
    }

    func antiCensorshipView() -> some View {
        VStack(alignment: .leading, spacing: 1) {
            SegmentedListItem(
                userInteraction: .enabled,
                accessibilityIdentifier: .vpnSettingsTableView,
                leading: {
                    SegmentedListItem(
                        isLastInList: false,
                        userInteraction: .enabled,
                        accessibilityIdentifier: censorshipSettings.first?.accessibilityIdentifier,
                        leading: {
                            itemFactory.leading(
                                for: .generic(title: censorshipSettings.first!.label))
                        },
                        trailing: {
                            itemFactory.trailing(for: .drillDown(title: "Automatic"))
                        }
                    )
                },
                groupedContent: {
                    SegmentedListItem(
                        userInteraction: .enabled,
                        accessibilityIdentifier: censorshipSettings.last?.accessibilityIdentifier,
                        leading: {
                            itemFactory.leading(
                                for: .generic(title: censorshipSettings.last!.label))
                        },
                        trailing: {
                            itemFactory.trailing(
                                for: .custom(items: [
                                    .button(
                                        icon: .info,
                                        onSelect: {
                                            print("hello")
                                        },
                                        sizing: .button
                                    ),
                                    .toggle(
                                        isOn: isQuantumResistanceEnabled,
                                        isDisabled: false
                                    ),
                                    .padding(),
                                ])
                            )
                        }
                    )
                }
            )
            .padding(.leading, UIMetrics.contentInsets.left)
            .padding(.trailing, UIMetrics.contentInsets.right)
        }
    }

    func DNSandIPSettingsView() -> some View {
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
                                for: .generic(title: DNSandIPSettings.first!.label))
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
                                for: .generic(title: DNSandIPSettings.last!.label))
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

#Preview {
    SettingsVPNSettingsView(viewModel: ObservabledVPNSettingsStub())
}
