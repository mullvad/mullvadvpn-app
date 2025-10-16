//
//  SettingsInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import UIKit

final class SettingsInteractor {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?
    var didUpdateSettings: (() -> Void)?

    private(set) var tunnelSettings: LatestTunnelSettings
    private(set) var deviceState: DeviceState

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        self.tunnelSettings = tunnelManager.settings
        self.deviceState = tunnelManager.deviceState

        let tunnelObserver =
            TunnelBlockObserver(
                didUpdateDeviceState: { [weak self] _, deviceState, _ in
                    guard let self = self else { return }
                    self.deviceState = deviceState
                    self.didUpdateSettings?()
                },
                didUpdateTunnelSettings: { [weak self] _, settings in
                    guard let self = self else { return }
                    self.tunnelSettings = settings
                    self.didUpdateSettings?()
                }
            )

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }
}
