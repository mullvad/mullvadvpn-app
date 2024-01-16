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

    var deviceState: DeviceState {
        tunnelManager.deviceState
    }

    init(
        tunnelManager: TunnelManager
    ) {
        self.tunnelManager = tunnelManager

        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] _, deviceState, _ in
                self?.didUpdateDeviceState?(deviceState)
            })

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }
}
