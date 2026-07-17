//
//  VPNSettingsView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2026-07-17.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct VPNSettingsView: View {
    private let itemFactory = SegmentedListItemFactory()

    let settingsInteractor: VPNSettingsInteractor
    let IPOverrideInteractor: IPOverrideInteractor
    let alertPresenter: AlertPresenter
    let navigationController: UINavigationController

    @Bindable var settings: ObservableVPNSettings
    var isQuantumResistanceEnabled: Binding<Bool>
    var wireGuardPort: Binding<WireGuardPort>
    /// When hotlinking settings from a pill in the `ConnectionView`, scroll automatically to the correct section
    let forceScrollTo: VPNSettingsSection

    @State var ipSectionIsExpanded: Bool = true
    @State private var alert: MullvadAlert?
    private let quantumResistantTunnelLabel = NSLocalizedString("Quantum-resistant tunnel", comment: "")

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
        ScrollViewReader { proxy in
            SettingsInfoContainerView {
                DNSandIPSettingsView()
                antiCensorshipView()
                quantumResistanceView()
                IPVersionView()
            }
            .onAppear {
                forceScrollToSection(proxy: proxy)
            }
        }
        .navigationTitle("VPN settings")
    }

    func forceScrollToSection(proxy: ScrollViewProxy) {
        switch forceScrollTo {
        case .quantumResistance:
            proxy.scrollTo(quantumResistantTunnelLabel)
        case .ipVersion:
            proxy.scrollTo(IPOptions.last?.id)
        case .none, .obfuscation:
            break
        }
    }

    // MARK: - DNS and IP Settings
    func DNSandIPSettingsView() -> some View {
        VStack(alignment: .leading, spacing: 1) {
            SegmentedListItem(
                isLastInList: false,
                accessibilityIdentifier: .dnsSettings,
                leadingAndTrailingDestination: {
                    GeometryReader { reader in
                        ScrollView {
                            DNSView(settingsInteractor: settingsInteractor, alertPresenter: alertPresenter)
                                .navigationTitle("DNS settings")
                                .frame(width: reader.size.width, height: reader.size.height)
                        }
                        .background(Color.mullvadBackground)
                    }
                },
                leading: {
                    itemFactory.leading(
                        for: .generic(title: NSLocalizedString("DNS settings", comment: "")))
                },
                trailing: {
                    itemFactory.trailing(for: .drillDown(title: ""))
                },
                groupedContent: {
                    SegmentedListItem(
                        accessibilityIdentifier: .ipOverrides,
                        leadingAndTrailingDestination: {
                            GeometryReader { reader in
                                ScrollView {
                                    IPOverrideView(
                                        ipOverrideInteractor: IPOverrideInteractor,
                                        alertPresenter: alertPresenter,
                                        navigationController: navigationController
                                    )
                                    .navigationTitle("Server IP override")
                                    .frame(width: reader.size.width, height: reader.size.height)
                                }
                                .background(Color.mullvadBackground)
                            }
                        },
                        leading: {
                            itemFactory.leading(
                                for: .generic(title: NSLocalizedString("Server IP override", comment: "")))
                        },
                        trailing: {
                            itemFactory.trailing(for: .drillDown(title: ""))
                        },
                    )
                },
            )
            .padding(.leading, UIMetrics.contentInsets.left)
            .padding(.trailing, UIMetrics.contentInsets.right)
        }
    }

    // MARK: - Anti censorship
    func antiCensorshipView() -> some View {
        VStack(alignment: .leading, spacing: 1) {
            SegmentedListItem(
                accessibilityIdentifier: .wireGuardPorts,
                leadingAndTrailingDestination: {
                    let viewModel = TunnelWireGuardPortSettingsViewModel(
                        port: wireGuardPort,
                        portRanges: settingsInteractor.cachedRelays?.relays.wireguard.portRanges ?? []
                    )
                    WireGuardPortSettingsView(
                        viewModel: viewModel,
                        options: [.automatic, .port51820, .port53]
                    )
                    .navigationTitle("WireGuard port")
                },
                leading: {
                    itemFactory.leading(
                        for: .generic(
                            title: "WireGuard port",
                            isSelected: false
                        )
                    )
                },
                trailing: {
                    itemFactory.trailing(
                        for: .drillDown(
                            title: WireGuardPort(
                                constraint: settings.tunnelSettings.relayConstraints.port
                            )
                            .description))
                },
                groupedContent: {
                    SegmentedListItem(
                        accessibilityIdentifier: .antiCensorship,
                        leadingAndTrailingDestination: {
                            AntiCensorshipView(
                                settingsInteractor: settingsInteractor,
                                settings: settings
                            )
                        },
                        leading: {
                            itemFactory.leading(
                                for: .generic(
                                    title: NSLocalizedString("Anti-censorship", comment: "")
                                )
                            )
                        },
                        trailing: {
                            itemFactory.trailing(
                                for: .drillDown(title: settings.tunnelSettings.wireGuardObfuscation.state.description))
                        }
                    )
                }
            )
            .padding(.leading, UIMetrics.contentInsets.left)
            .padding(.trailing, UIMetrics.contentInsets.right)
        }
    }

    func getQuantumResistanceAlert(completion: @escaping () -> Void) -> MullvadAlert {
        MullvadAlert(
            type: .info,
            messages: [
                "This feature makes the WireGuard tunnel resistant to potential attacks from quantum computers.",
                """
                It does this by performing an extra key exchange using a quantum safe algorithm and mixing the result into WireGuard’s regular encryption. This extra step uses approximately 500 kiB of traffic every time a new tunnel is established.
                """,
            ], customView: nil,
            actions: [
                MullvadAlert.Action(type: .primary, title: "Got it!", handler: completion)
            ])
    }

    // MARK: - Quantum Resistance

    func quantumResistanceView() -> some View {
        QuantumResistanceView(isQuantumResistanceEnabled: isQuantumResistanceEnabled)
            .id(quantumResistantTunnelLabel)
            .padding(.leading, UIMetrics.contentInsets.left)
            .padding(.trailing, UIMetrics.contentInsets.right)
    }

    // MARK: - IP version
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
                            isSelected: settings.tunnelSettings.ipVersion == option.id))
                },
                onSelect: {
                    settings.tunnelSettings.ipVersion = option.id
                    settingsInteractor.tunnelManager.updateSettings([.ipVersion(settings.tunnelSettings.ipVersion)])
                }
            )
            .id(option.id)
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
                MullvadAlert.Action(type: .primary, title: "Got it!", handler: completion)
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
                    // Align this info button with the quantum resistance section one
                    .padding(.trailing, UIMetrics.contentInsets.right + 6)
                },
                segment: {
                    itemFactory.segment(
                        for: .expand(
                            isExpanded: $ipSectionIsExpanded.wrappedValue,
                            onSelect: {
                                withAnimation(.default.speed(3)) {
                                    $ipSectionIsExpanded.wrappedValue.toggle()
                                }
                            }))
                },
                groupedContent: {
                    $ipSectionIsExpanded.wrappedValue ? IPVersionSelectionView() : nil
                }
            )
            .mullvadAlert(item: $alert)
            .padding(.leading, UIMetrics.contentInsets.left)
            .padding(.trailing, UIMetrics.contentInsets.right)
        }
    }
}
