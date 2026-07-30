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
    @State var navigationPath = NavigationPath()
    var actorObserver: MainActorObserver?

    init(
        settingsInteractor: VPNSettingsInteractor, IPOverrideInteractor: IPOverrideInteractor,
        alertPresenter: AlertPresenter, navigationPath: NavigationPath = NavigationPath()
    ) {
        self.settingsInteractor = settingsInteractor
        self.IPOverrideInteractor = IPOverrideInteractor
        self.alertPresenter = alertPresenter
        self.navigationPath = navigationPath

        self.observableSettings = ObservableVPNSettings(tunnelSettings: settingsInteractor.tunnelManager.settings)
        self.actorObserver = MainActorObserver(
            observable: observableSettings, tunnelManager: settingsInteractor.tunnelManager)
        actorObserver?.start()
    }

    var body: some View {
        SettingsVPNSettingsView(
            settingsInteractor: settingsInteractor,
            IPOverrideInteractor: IPOverrideInteractor,
            alertPresenter: alertPresenter,
            viewModel: observableSettings
        )
        .background(Color(.secondaryColor))
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
