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
    let alertPresenter: AlertPresenter
    @State var navigationPath = NavigationPath()

    var body: some View {
        NavigationStack(path: $navigationPath) {
            SettingsVPNSettingsView(
                viewModel: ObservabledVPNSettingsStub(), path: $navigationPath
            )
            .navigationDestination(for: SettingsDestinationView.self) { path in
                destinationView(path)
            }
            .background(Color(.secondaryColor))
        }
    }

    @ViewBuilder
    func destinationView(_ path: SettingsDestinationView) -> some View {
        switch path {
        case .antiCensorship: AntiCensorshipView()
        case .dnsSettings:
            DNSView(settingsInteractor: settingsInteractor, alertPresenter: alertPresenter)
                .navigationTitle("DNS Settings")
        case .serverIPOverride: Text("Server IP Override")
        }
    }
}

enum SettingsDestinationView: Codable {
    case dnsSettings
    case serverIPOverride
    case antiCensorship
}
