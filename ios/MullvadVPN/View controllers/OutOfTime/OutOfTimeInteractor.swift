//
//  OutOfTimeInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations
@preconcurrency import StoreKit

final class OutOfTimeInteractor: Sendable {
    private let tunnelManager: TunnelManager

    nonisolated(unsafe) private var tunnelObserver: TunnelObserver?

    nonisolated(unsafe) private let logger = Logger(label: "OutOfTimeInteractor")

    private let accountUpdateTimerInterval: Duration = .minutes(1)
    nonisolated(unsafe) private var accountUpdateTimer: DispatchSourceTimer?

    nonisolated(unsafe) var didReceiveTunnelStatus: (@Sendable (TunnelStatus) -> Void)?
    nonisolated(unsafe) var didAddMoreCredit: (@Sendable () -> Void)?

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager

        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                self?.didReceiveTunnelStatus?(tunnelStatus)
            },
            didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                let isInactive = previousDeviceState.accountData?.isExpired == true
                let isActive = deviceState.accountData?.isExpired == false
                if isInactive && isActive {
                    self?.didAddMoreCredit?()
                }
            }
        )

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }

    var tunnelStatus: TunnelStatus {
        tunnelManager.tunnelStatus
    }

    var deviceState: DeviceState {
        tunnelManager.deviceState
    }

    func stopTunnel() {
        tunnelManager.stopTunnel()
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

        timer.schedule(
            wallDeadline: .now() + accountUpdateTimerInterval,
            repeating: accountUpdateTimerInterval.timeInterval
        )
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
