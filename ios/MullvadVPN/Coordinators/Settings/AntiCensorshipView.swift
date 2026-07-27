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
    let settings: ObservableVPNSettings
    let itemFactory = SegmentedListItemFactory()
    let availableObfuscations = WireGuardObfuscationState.allCases

    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                Text(
                    """
                    These methods may be useful in situations where you are blocked from reaching Mullvad. When "Automatic" is selected, the app will attempt all methods until one works.
                    """
                )
                .foregroundStyle(Color.mullvadTextPrimary)
                Spacer()
                Text(
                    """
                    Please note that these methods do not improve performance, and may increase system utilization and battery consumption.
                    """
                )
                .foregroundStyle(Color.mullvadTextPrimary)
                .bold()
                Spacer()
                SegmentedListItem(
                    userInteraction: .enabled,
                    leading: {
                        itemFactory.leading(
                            for:
                                .generic(title: "Method")
                        )
                    },
                    groupedContent: {
                        ForEach(Array(availableObfuscations.enumerated()), id: \.element.description) { index, option in
                            SegmentedListItem(
                                level: 1,
                                isLastInList: index == availableObfuscations.count - 1,
                                accessibilityIdentifier: .udpOverTcpObfuscationSettings,  // ???
                                leading: {
                                    leadingView(for: option)
                                },
                                segment: {
                                    if [
                                        WireGuardObfuscationState.automatic, WireGuardObfuscationState.quic,
                                        WireGuardObfuscationState.off,
                                    ].contains(option) {
                                        EmptyView()
                                    } else {
                                        itemFactory.segment(
                                            for: .expand(
                                                isExpanded: false,
                                                onSelect: {
                                                    print("Selected \(option)")
                                                })
                                        )
                                        .rotationEffect(.degrees(-90))
                                    }
                                },
                                onSelect: {
                                    // Change the iteration here to match real wireguard obfuscation
                                }
                            )
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

    // How to handle better wireguard port ???
    @ViewBuilder
    func leadingView(for state: WireGuardObfuscationState) -> some View {
        let obfuscation = settings.tunnelSettings.wireGuardObfuscation
        let title = state.description
        let subtitle: String? =
            switch state {
            case .shadowsocks: "Port: \(obfuscation.shadowsocksPort)"
            case .lwo: "Port \(obfuscation.lwoPort)"
            case .udpOverTcp: "Port \(obfuscation.lwoPort)"
            default: nil
            }

        let isSelected = state == obfuscation.state
        itemFactory.leading(for: .generic(title: title, subtitle: subtitle, level: 1, isSelected: isSelected))
    }

    @ViewBuilder
    func obfuscationView(_ state: WireGuardObfuscationState) -> some View {
        switch state {
        case .shadowsocks:
            let viewModel = TunnelShadowsocksObfuscationSettingsViewModel(
                tunnelManager: settingsInteractor.tunnelManager)
            ShadowsocksObfuscationSettingsView(viewModel: viewModel)
        default: Text(state.description)
        }
    }
}
