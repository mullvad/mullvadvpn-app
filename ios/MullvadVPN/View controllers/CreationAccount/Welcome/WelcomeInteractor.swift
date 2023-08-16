//
//  WelcomeInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-29.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging

class WelcomeInteractor {
    private let logger = Logger(label: "WelcomeInteractor")

    private let accountUpdateTimerInterval: TimeInterval = 60
    private var accountUpdateTimer: DispatchSourceTimer?

    private let deviceData: StoredDeviceData
    private let accountData: StoredAccountData

    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?

    var didUpdateDeviceState: ((DeviceState) -> Void)?

    var viewModel: WelcomeViewModel {
        WelcomeViewModel(
            deviceName: deviceData.capitalizedName,
            accountNumber: accountData.number.formattedAccountNumber
        )
    }

    init(deviceData: StoredDeviceData, accountData: StoredAccountData, tunnelManager: TunnelManager) {
        self.deviceData = deviceData
        self.accountData = accountData
        self.tunnelManager = tunnelManager

        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] tunnelManager, deviceState, previousDeviceState in
                self?.didUpdateDeviceState?(deviceState)
            })

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver
    }

    func startAccountUpdateTimer() {
        logger.debug(
            "Start polling account updates every \(accountUpdateTimerInterval) second(s)."
        )

        let timer = DispatchSource.makeTimerSource(queue: .main)
        timer.setEventHandler { [weak self] in
            self?.tunnelManager.updateAccountData()
        }

        accountUpdateTimer?.cancel()
        accountUpdateTimer = timer

        timer.schedule(wallDeadline: .now() + accountUpdateTimerInterval, repeating: accountUpdateTimerInterval)
        timer.activate()
    }

    func stopAccountUpdateTimer() {
        logger.debug(
            "Stop polling account updates."
        )

        accountUpdateTimer?.cancel()
        accountUpdateTimer = nil
    }
}
