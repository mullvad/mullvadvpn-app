// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations

final class OutOfTimeInteractor {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?

    private let logger = Logger(label: "OutOfTimeInteractor")

    private let accountUpdateTimerInterval: Duration = .minutes(1)
    private var accountUpdateTimer: DispatchSourceTimer?

    private var tunnelStatusCallbacks: [((TunnelStatus) -> Void)?] = []
    var didAddMoreCredit: (() -> Void)?

    private let accountDataPoller: AccountDataPoller

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager
        accountDataPoller = AccountDataPoller(logger: logger, tunnelManager: tunnelManager)

        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                self?.tunnelStatusCallbacks.forEach { callback in
                    callback?(tunnelStatus)
                }
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

    func didReceiveTunnelStatus(_ callback: ((TunnelStatus) -> Void)?) {
        tunnelStatusCallbacks.append(callback)
    }

    func stopTunnel() {
        tunnelManager.stopTunnel()
    }

    func startAccountUpdateTimer() {
        accountDataPoller.startAccountUpdateTimer()
    }

    func stopAccountUpdateTimer() {
        accountDataPoller.stopAccountUpdateTimer()
    }
}
