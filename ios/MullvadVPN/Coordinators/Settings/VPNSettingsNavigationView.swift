// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes
import SwiftUI

struct VPNSettingsNavigationView: View {
    let settingsInteractor: VPNSettingsInteractor
    let IPOverrideInteractor: IPOverrideInteractor
    let alertPresenter: AlertPresenter
    let navigationController: UINavigationController
    @Bindable var observableSettings: ObservableVPNSettings
    let presentOnlySection: VPNSettingsSection

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

    var wireGuardPort: Binding<WireGuardPort> {
        Binding<WireGuardPort>(
            get: {
                WireGuardPort(
                    constraint:
                        observableSettings.tunnelSettings.relayConstraints.port
                )
            },
            set: { newPort in
                let newPortConstraint = RelayConstraint<UInt16>(newPort)
                observableSettings.tunnelSettings.relayConstraints.port = newPortConstraint
                var relayConstraints = settingsInteractor.tunnelManager.settings.relayConstraints
                relayConstraints.port = newPortConstraint
                settingsInteractor.tunnelManager.updateSettings([.relayConstraints(relayConstraints)])
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
            VPNSettingsView(
                settingsInteractor: settingsInteractor,
                IPOverrideInteractor: IPOverrideInteractor,
                alertPresenter: alertPresenter,
                navigationController: navigationController,
                settings: observableSettings,
                isQuantumResistanceEnabled: isQuantumResistanceEnabled,
                wireGuardPort: wireGuardPort,
                forceScrollTo: presentOnlySection
            )
            .background(Color(.secondaryColor))
        }
    }
}
