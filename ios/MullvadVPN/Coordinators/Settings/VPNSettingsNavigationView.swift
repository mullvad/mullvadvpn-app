//
//  VPNSettingsNavigationView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2026-07-22.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI

actor MainActorObserver {
    let observable: ObservableVPNSettings
    let tunnelManager: TunnelManager

    init(observable: ObservableVPNSettings, tunnelManager: TunnelManager) {
        self.observable = observable
        self.tunnelManager = tunnelManager
    }

    @MainActor
    func start() {
        _ = withObservationTracking {
            Task {
                print("Settings have changed: \(observable.tunnelSettings)")
            }
        } onChange: {
            Task {
                await MainActor.run {
                    self.start()
                }
            }
        }
    }
}

struct VPNSettingsNavigationView: View {
    let settingsInteractor: VPNSettingsInteractor
    let IPOverrideInteractor: IPOverrideInteractor
    let alertPresenter: AlertPresenter
    @Bindable var observableSettings: ObservableVPNSettings
    var actorObserver: MainActorObserver?
    var presentOnlySection: VPNSettingsSection

    init(
        settingsInteractor: VPNSettingsInteractor, IPOverrideInteractor: IPOverrideInteractor,
        alertPresenter: AlertPresenter, presentOnlySection: VPNSettingsSection
    ) {
        self.settingsInteractor = settingsInteractor
        self.IPOverrideInteractor = IPOverrideInteractor
        self.alertPresenter = alertPresenter
        self.presentOnlySection = presentOnlySection

        self.observableSettings = ObservableVPNSettings(tunnelSettings: settingsInteractor.tunnelManager.settings)
        self.actorObserver = MainActorObserver(
            observable: observableSettings, tunnelManager: settingsInteractor.tunnelManager)
        actorObserver?.start()
    }

    var body: some View {
        switch presentOnlySection {
        case .obfuscation:
            destinationView(.antiCensorship)
        case .none, .quantumResistance, .ipVersion: //TODO: Handle quantum resistance and IP version
            SettingsVPNSettingsView(
                settingsInteractor: settingsInteractor,
                IPOverrideInteractor: IPOverrideInteractor,
                alertPresenter: alertPresenter,
                viewModel: observableSettings
            )
            .background(Color(.secondaryColor))
        }
    }

    // Can this be a generic way to not have to repeat navigation configuration each time ?
    @ViewBuilder
    func destinationView(_ path: SettingsDestinationView) -> some View {
        switch path {
        case .antiCensorship:
            AntiCensorshipView(
                settingsInteractor: settingsInteractor,
                settings: observableSettings)
        case .dnsSettings:
            DNSView(settingsInteractor: settingsInteractor, alertPresenter: alertPresenter)
                .navigationTitle("DNS Settings")
        case .serverIPOverride:
            IPOverrideView(ipOverrideInteractor: IPOverrideInteractor, alertPresenter: alertPresenter)
                .navigationTitle("Server IP override")
        case .shadowsocks:
            ShadowsocksObfuscationSettingsView(
                port: $observableSettings.tunnelSettings.wireGuardObfuscation.shadowsocksPort
            )
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
