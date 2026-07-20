//
//  SettingsVPNSettingsView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2026-07-17.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
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

    @State var isExpanded: Bool = true
    @State private var alert: MullvadAlert?

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

    private struct IPSettingOption: Identifiable {
        let id: IPVersion
        let label: String
        let accessibilityIdentifier: AccessibilityIdentifier
    }

    private let IPOptions: [IPSettingOption] = [
        .init(
            id: .automatic, label: NSLocalizedString("Automatic", comment: ""),
            accessibilityIdentifier: .ipVersionAutomatic),
        .init(id: .ipv4, label: NSLocalizedString("IPv4", comment: ""), accessibilityIdentifier: .ipVersionIPv4),
        .init(id: .ipv6, label: NSLocalizedString("IPv6", comment: ""), accessibilityIdentifier: .ipVersionIPv6),
    ]

    var body: some View {
        SettingsInfoContainerView {
            DNSandIPSettingsView()
            antiCensorshipView()
            IPVersionView()
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

    func IPVersionSelectionView() -> some View {
        ForEach(Array(IPOptions.enumerated()), id: \.element.id) { index, option in
            SegmentedListItem(
                level: 1,
                isLastInList: index == IPOptions.count - 1,
                accessibilityIdentifier: option.accessibilityIdentifier,
                leading: {
                    itemFactory.leading(
                        for: .generic(
                            title: option.label, level: 1,
                            isSelected: viewModel.ipVersion == option.id))
                },
                onSelect: {
                    viewModel.ipVersion = option.id
                }
            )
        }
    }

    func getIPVersionAlert(completion: @escaping () -> Void) -> MullvadAlert {
        MullvadAlert(
            type: .info,
            messages: [
                """
                This setting controls whether the app connects to VPN servers using IPv4 or IPv6.Automatic setting allows app to choose between both, but currently app will only use IPv4.
                """
            ], customView: nil,
            actions: [
                MullvadAlert.Action(type: .default, title: "Got it!", handler: completion)
            ])
    }

    func IPVersionView() -> some View {
        VStack(alignment: .leading, spacing: 0) {
            SegmentedListItem(
                userInteraction: .enabledWithoutHighlight,
                accessibilityIdentifier: .ipVersionCell,
                leading: {
                    itemFactory.leading(for: .generic(title: NSLocalizedString("IP version", comment: "")))
                },
                trailing: {
                    itemFactory.segment(
                        for: .info(onSelect: {
                            alert = getIPVersionAlert(completion: { alert = nil })
                        })
                    )
                    .padding(.trailing, UIMetrics.contentInsets.right)
                },
                segment: {
                    itemFactory.segment(
                        for: .expand(
                            isExpanded: $isExpanded.wrappedValue,
                            onSelect: {
                                $isExpanded.wrappedValue.toggle()
                            }))
                },
                groupedContent: {
                    $isExpanded.wrappedValue ? IPVersionSelectionView() : nil
                }
            )
            .padding(.leading, UIMetrics.contentInsets.left)
            .padding(.trailing, UIMetrics.contentInsets.right)
        }
        .mullvadAlert(item: $alert)
    }

}

#Preview {
    SettingsVPNSettingsView(viewModel: ObservabledVPNSettingsStub())
}
