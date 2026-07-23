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
    let itemFactory = SegmentedListItemFactory()
    let availableObfuscations = WireGuardObfuscationState.allCases
    @Binding var path: NavigationPath

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
                                    itemFactory.leading(
                                        for: .generic(
                                            title: option.description,
                                            subtitle: "Port: Automatic",  // ???
                                            level: 1, isSelected: false))  // ???
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
                                                    navigateToObfuscationSetting(for: option)
                                                })
                                        )
                                        .rotationEffect(.degrees(-90))
                                    }
                                },
                                onSelect: {
                                    print("Selected obfuscation")
                                }
                            )
                        }
                    }
                )
            }
            .padding(.leading, UIMetrics.contentInsets.left)
            .padding(.trailing, UIMetrics.contentInsets.right)
        }
        .background(Color(.secondaryColor))
    }

    func navigateToObfuscationSetting(for state: WireGuardObfuscationState) {
        switch state {
        case .automatic, .quic, .off, .on: break
        case .shadowsocks: path.append(SettingsDestinationView.shadowsocks)
        case .udpOverTcp: path.append(SettingsDestinationView.udpOverTcp)
        case .lwo: path.append(SettingsDestinationView.lwo)
        }
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
