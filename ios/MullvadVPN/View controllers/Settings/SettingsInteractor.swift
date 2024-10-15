//
//  SettingsInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

final class SettingsInteractor {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?

    var didUpdateDeviceState: ((DeviceState) -> Void)?

    var tunnelSettings: LatestTunnelSettings {
        tunnelManager.settings
    }

    var deviceState: DeviceState {
        tunnelManager.deviceState
    }

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager

        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] _, deviceState, _ in
                self?.didUpdateDeviceState?(deviceState)
            })

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }

    func updateDAITASettings(_ settings: DAITASettings) {
        tunnelManager.updateSettings([.daita(settings)])
    }

    func evaluateDaitaSettingsCompatibility(_ settings: DAITASettings) -> DAITASettingsCompatibilityError? {
        guard settings.daitaState.isEnabled else { return nil }

        var tunnelSettings = tunnelSettings
        tunnelSettings.daita = settings

        // Return error if no relays could be selected.
        guard (try? tunnelManager.selectRelays(tunnelSettings: tunnelSettings)) != nil else {
            return tunnelSettings.tunnelMultihopState.isEnabled ? .multihop : .singlehop
        }

        return nil
    }
}
