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

    struct Option: Identifiable {
        typealias ID = String
        let state: WireGuardObfuscationState
        let accessibilityIdentifier: AccessibilityIdentifier

        var id: ID {
            state.description
        }
    }

    init(settingsInteractor: VPNSettingsInteractor, settings: ObservableVPNSettings) {
        self.settingsInteractor = settingsInteractor
        self.settings = settings
    }

    let availableObfuscations: [Option] =
        [
            .init(state: .automatic, accessibilityIdentifier: .wireGuardObfuscationAutomatic),
            .init(state: .lwo, accessibilityIdentifier: .wireGuardObfuscationLwo),
            .init(state: .quic, accessibilityIdentifier: .wireGuardObfuscationQuic),
            .init(state: .shadowsocks, accessibilityIdentifier: .wireGuardObfuscationShadowsocks),
            .init(state: .udpOverTcp, accessibilityIdentifier: .wireGuardObfuscationUdpOverTcp),
            .init(state: .off, accessibilityIdentifier: .wireGuardObfuscationOff),
        ]

    var body: some View {
        ScrollViewReader { proxy in
            antiCensorshipViewContents()
                .onAppear {
                    forceScrollToSection(proxy: proxy)
                }
        }
        .navigationTitle("Anti-censorship")
        .background(Color(.secondaryColor))
    }

    @ViewBuilder
    private func antiCensorshipViewContents() -> some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 0) {
                headerText()
                    .padding(.bottom, UIMetrics.contentInsets.bottom)
                SegmentedListItem(
                    userInteraction: .enabledWithoutHighlight,
                    leading: {
                        itemFactory.leading(
                            for:
                                .generic(title: NSLocalizedString("Method", comment: ""))
                        )
                    },
                    groupedContent: {
                        Group {
                            ForEach(Array(availableObfuscations.enumerated()), id: \.element.id) {
                                (index: Int, option: Option) in
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
                                .id(option.id)
                            }
                        }
                    }
                )
                .onChange(of: settings.tunnelSettings.wireGuardObfuscation, initial: false) { oldValue, newValue in
                    // Prevent spamming updates if the same cell is pressed multiple times in a row
                    if oldValue != newValue {
                        updateSettings()
                    }
                }
            }
            .padding(.leading, UIMetrics.contentInsets.left)
            .padding(.trailing, UIMetrics.contentInsets.right)
            .padding(.bottom, UIMetrics.contentInsets.bottom)
        }
    }

    func forceScrollToSection(proxy: ScrollViewProxy) {
        proxy.scrollTo(settings.tunnelSettings.wireGuardObfuscation.state.description)
    }

    func updateSettings() {
        settingsInteractor.tunnelManager.updateSettings([
            .obfuscation(settings.tunnelSettings.wireGuardObfuscation)
        ])
    }

    @ViewBuilder
    private func headerText() -> some View {
        Text(
            """
            These methods may be useful in situations where you are blocked from reaching Mullvad. When "Automatic" is selected, the app will attempt all methods until one works.
            """
        )
        .font(.mullvadTiny)
        .foregroundStyle(Color.mullvadTextSecondary)
        .padding(.bottom, 8)
        Text(
            """
            Please note that these methods do not improve performance, and may increase system utilization and battery consumption.
            """
        )
        .font(.mullvadTinySemiBold)
        .foregroundStyle(Color.mullvadTextSecondary)
    }

    @ViewBuilder
    private func leadingView(for state: WireGuardObfuscationState) -> some View {
        let obfuscation = settings.tunnelSettings.wireGuardObfuscation
        let title = state.description
        let formatPort: (String) -> String = { port in
            String(format: NSLocalizedString("Port: %@", comment: ""), port)
        }
        let subtitle: String? =
            switch state {
            case .shadowsocks: formatPort(obfuscation.shadowsocksPort.description)
            case .lwo: formatPort(obfuscation.lwoPort.description)
            case .udpOverTcp: formatPort(obfuscation.udpOverTcpPort.description)
            default: nil
            }

        let isSelected = state == obfuscation.state
        itemFactory.leading(for: .generic(title: title, subtitle: subtitle, level: 1, isSelected: isSelected))
    }

    @ViewBuilder
    private func segmentView(for state: WireGuardObfuscationState) -> some View {
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
                    .onDisappear {
                        updateSettings()
                    }
                case .udpOverTcp:
                    UDPOverTCPObfuscationSettingsView(
                        port: $settings.tunnelSettings.wireGuardObfuscation.udpOverTcpPort
                    )
                    .navigationTitle("UDP-over-TCP")
                    .onDisappear {
                        updateSettings()
                    }
                case .lwo:
                    let viewModel = TunnelLwoObfuscationSettingsViewModel(
                        port: $settings.tunnelSettings.wireGuardObfuscation.lwoPort,
                        portRanges: settingsInteractor.cachedRelays?.relays.wireguard.portRanges ?? [])
                    LwoObfuscationSettingsView(
                        viewModel: viewModel
                    )
                    .navigationTitle("LWO")
                    .onDisappear {
                        updateSettings()
                    }
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
