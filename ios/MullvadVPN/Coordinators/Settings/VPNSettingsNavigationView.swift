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
    let navigationController: UINavigationController
    @Bindable var observableSettings: ObservableVPNSettings
    var actorObserver: MainActorObserver?
    var presentOnlySection: VPNSettingsSection

    init(
        settingsInteractor: VPNSettingsInteractor, IPOverrideInteractor: IPOverrideInteractor,
        alertPresenter: AlertPresenter,
        navigationController: UINavigationController,
        presentOnlySection: VPNSettingsSection
    ) {
        self.settingsInteractor = settingsInteractor
        self.IPOverrideInteractor = IPOverrideInteractor
        self.alertPresenter = alertPresenter
        self.navigationController = navigationController
        self.presentOnlySection = presentOnlySection

        self.observableSettings = ObservableVPNSettings(tunnelSettings: settingsInteractor.tunnelManager.settings)
        self.actorObserver = MainActorObserver(
            observable: observableSettings, tunnelManager: settingsInteractor.tunnelManager)
        actorObserver?.start()
    }

    var isQuantumResistanceEnabled: Binding<Bool> {
        Binding<Bool>(
            get: {
                observableSettings.tunnelSettings.tunnelQuantumResistance.isEnabled
            },
            set: { enabled in
                observableSettings.tunnelSettings.tunnelQuantumResistance = enabled ? .on : .off
                settingsInteractor.tunnelManager.updateSettings([
                    .quantumResistance(observableSettings.tunnelSettings.tunnelQuantumResistance)
                ])
            }
        )
    }

    var body: some View {
        switch presentOnlySection {
        case .obfuscation:
            AntiCensorshipView(
                settingsInteractor: settingsInteractor,
                settings: observableSettings)
        case .none, .quantumResistance, .ipVersion:
            SettingsVPNSettingsView(
                settingsInteractor: settingsInteractor,
                IPOverrideInteractor: IPOverrideInteractor,
                alertPresenter: alertPresenter,
                navigationController: navigationController,
                viewModel: observableSettings,
                isQuantumResistanceEnabled: isQuantumResistanceEnabled,
                forceScrollTo: presentOnlySection
            )
            .background(Color(.secondaryColor))
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
