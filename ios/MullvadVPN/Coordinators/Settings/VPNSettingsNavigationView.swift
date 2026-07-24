//
//  VPNSettingsNavigationView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2026-07-22.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct VPNSettingsNavigationView: View {
    let settingsInteractor: VPNSettingsInteractor
    let IPOverrideInteractor: IPOverrideInteractor
    let alertPresenter: AlertPresenter
    @State var navigationPath = NavigationPath()

    var body: some View {
        NavigationStack(path: $navigationPath) {
            SettingsVPNSettingsView(
                viewModel: ObservableVPNSettings(), path: $navigationPath
            )
            .navigationDestination(for: SettingsDestinationView.self) { path in
                destinationView(path)
            }
            .background(Color(.secondaryColor))
        }
    }

    // Can this be a generic way to not have to repeat navigation configuration each time ?
    @ViewBuilder
    func destinationView(_ path: SettingsDestinationView) -> some View {
        switch path {
        case .antiCensorship: AntiCensorshipView(settingsInteractor: settingsInteractor, path: $navigationPath)
        case .dnsSettings:
            DNSView(settingsInteractor: settingsInteractor, alertPresenter: alertPresenter)
                .navigationTitle("DNS Settings")
        case .serverIPOverride:
            IPOverrideView(ipOverrideInteractor: IPOverrideInteractor, alertPresenter: alertPresenter)
                .navigationTitle("Server IP override")
        case .shadowsocks:
            let viewModel = TunnelShadowsocksObfuscationSettingsViewModel(
                tunnelManager: settingsInteractor.tunnelManager)
            ShadowsocksObfuscationSettingsView(viewModel: viewModel)
                .navigationTitle("Shadowsocks")
                .navigationBarTitleDisplayMode(.large)
        case .lwo:
            let viewModel = TunnelLwoObfuscationSettingsViewModel(
                tunnelManager: settingsInteractor.tunnelManager,
                portRanges: settingsInteractor.cachedRelays?.relays.wireguard.portRanges ?? [])
            LwoObfuscationSettingsView(viewModel: viewModel)
        case .udpOverTcp:
            let viewModel = TunnelUDPOverTCPObfuscationSettingsViewModel(
                tunnelManager: settingsInteractor.tunnelManager)
            UDPOverTCPObfuscationSettingsView(viewModel: viewModel)
        }
    }
}

enum SettingsDestinationView: Codable {
    case dnsSettings
    case serverIPOverride
    case antiCensorship
    case shadowsocks
    case lwo
    case udpOverTcp
}
