//
//  SettingsInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
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

        var compatibilityError: DAITASettingsCompatibilityError?

        do {
            _ = try tunnelManager.selectRelays(tunnelSettings: tunnelSettings)
        } catch let error as NoRelaysSatisfyingConstraintsError where error.reason == .noDaitaRelaysFound {
            // Return error if no relays could be selected due to DAITA constraints.
            compatibilityError = tunnelSettings.tunnelMultihopState.isEnabled ? .multihop : .singlehop
        } catch let error as NoRelaysSatisfyingConstraintsError {
            // Even if the constraints error is not DAITA specific, if both DAITA and Direct only are enabled,
            // we should return a DAITA related error since the current settings would have resulted in the
            // relay selector not being able to select a DAITA relay anyway.
            if settings.isDirectOnly {
                compatibilityError = tunnelSettings.tunnelMultihopState.isEnabled ? .multihop : .singlehop
            }
        } catch {}

        return compatibilityError
    }
}
