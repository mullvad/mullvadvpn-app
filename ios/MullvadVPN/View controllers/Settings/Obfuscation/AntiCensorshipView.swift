//
//  AntiCensorshipView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2026-07-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

struct AntiCensorshipView: View {
    let settingsInteractor: VPNSettingsInteractor
    @Bindable var settings: ObservableVPNSettings
    let itemFactory = SegmentedListItemFactory()

    struct Options: Identifiable {
        typealias ID = String
        let state: WireGuardObfuscationState
        let accessibilityIdentifier: AccessibilityIdentifier

        var id: ID {
            state.description
        }
    }

    init(settingsInteractor: VPNSettingsInteractor) {
        self.settingsInteractor = settingsInteractor
        self.settings = ObservableVPNSettings(tunnelSettings: settingsInteractor.tunnelManager.settings)
    }

    // TODO: Add an accessibility identifier for segment views
    let availableObfuscations: [Options] =
        [
            .init(state: .automatic, accessibilityIdentifier: .wireGuardObfuscationAutomatic),
            .init(state: .shadowsocks, accessibilityIdentifier: .wireGuardObfuscationShadowsocks),
            .init(state: .udpOverTcp, accessibilityIdentifier: .wireGuardObfuscationUdpOverTcp),
            .init(state: .quic, accessibilityIdentifier: .wireGuardObfuscationQuic),
            .init(state: .lwo, accessibilityIdentifier: .wireGuardObfuscationLwo),
            .init(state: .off, accessibilityIdentifier: .wireGuardObfuscationOff),
        ]

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                headerText()
                Spacer()
                SegmentedListItem(
                    userInteraction: .enabledWithoutHighlight,
                    leading: {
                        itemFactory.leading(
                            for:
                                .generic(title: "Method")
                        )
                    },
                    groupedContent: {
                        Group {
                            ForEach(Array(availableObfuscations.enumerated()), id: \.element.id) {
                                (index: Int, option: Options) in
                                let state = option.state
                                let accessibilityIdentifier = option.accessibilityIdentifier
                                SegmentedListItem(
                                    level: 1,
                                    isLastInList: index == availableObfuscations.count - 1,
                                    accessibilityIdentifier: accessibilityIdentifier,
                                    leading: {
                                        leadingView(for: state)
                                    },
                                    segment: {
                                        segmentView(for: state)
                                    },
                                    onSelect: {
                                        settings.tunnelSettings.wireGuardObfuscation.state = state
                                    }
                                )
                            }
                        }
                        .onChange(of: settings.tunnelSettings.wireGuardObfuscation) {
                            settingsInteractor.tunnelManager.updateSettings([
                                .obfuscation(settings.tunnelSettings.wireGuardObfuscation)
                            ])
                        }
                    }
                )
            }
            .padding(.leading, UIMetrics.contentInsets.left)
            .padding(.trailing, UIMetrics.contentInsets.right)
        }
        .navigationTitle("Anti-censorship")
        .background(Color(.secondaryColor))
    }

    @ViewBuilder
    func headerText() -> some View {
        Text(
            """
            These methods may be useful in situations where you are blocked from reaching Mullvad. When "Automatic" is selected, the app will attempt all methods until one works.
            """
        )
        .font(.mullvadTiny)
        .foregroundStyle(Color.mullvadTextSecondary)
        Spacer()
        Spacer()
        Text(
            """
            Please note that these methods do not improve performance, and may increase system utilization and battery consumption.
            """
        )
        .font(.mullvadTinySemiBold)
        .foregroundStyle(Color.mullvadTextSecondary)
    }

    @ViewBuilder
    func leadingView(for state: WireGuardObfuscationState) -> some View {
        let obfuscation = settings.tunnelSettings.wireGuardObfuscation
        let title = state.description
        let subtitle: String? =
            switch state {
            case .shadowsocks: "Port: \(obfuscation.shadowsocksPort)"
            case .lwo: "Port: \(obfuscation.lwoPort)"
            case .udpOverTcp: "Port: \(obfuscation.udpOverTcpPort)"
            default: nil
            }

        let isSelected = state == obfuscation.state
        itemFactory.leading(for: .generic(title: title, subtitle: subtitle, level: 1, isSelected: isSelected))
    }

    @ViewBuilder
    func segmentView(for state: WireGuardObfuscationState) -> some View {
        if [WireGuardObfuscationState.automatic, WireGuardObfuscationState.quic, WireGuardObfuscationState.off]
            .contains(state)
        {
            EmptyView()
        } else {
            NavigationLink {
                switch state {
                case .shadowsocks:
                    ShadowsocksObfuscationSettingsView(
                        port: $settings.tunnelSettings.wireGuardObfuscation.shadowsocksPort
                    )
                    .navigationTitle("Shadowsocks")
                case .udpOverTcp:
                    UDPOverTCPObfuscationSettingsView(
                        port: $settings.tunnelSettings.wireGuardObfuscation.udpOverTcpPort
                    )
                    .navigationTitle("UDP-over-TCP")
                case .lwo:
                    let viewModel = TunnelLwoObfuscationSettingsViewModel(
                        portRanges: settingsInteractor.cachedRelays?.relays.wireguard.portRanges ?? [])
                    LwoObfuscationSettingsView(
                        viewModel: viewModel, port: $settings.tunnelSettings.wireGuardObfuscation.lwoPort
                    )
                    .navigationTitle("LWO")
                default: EmptyView()
                }
            } label: {
                switch state {
                case .shadowsocks:
                    itemFactory.image(for: .chevron)
                        .accessibilityIdentifier(.wireGuardObfuscationShadowsocksPort)
                case .udpOverTcp:
                    itemFactory.image(for: .chevron)
                        .accessibilityIdentifier(.wireGuardObfuscationUdpOverTcpPort)
                case .lwo:
                    itemFactory.image(for: .chevron)
                        .accessibilityIdentifier(.wireGuardObfuscationLwoPort)
                default: EmptyView()
                }
            }
        }
    }
}
