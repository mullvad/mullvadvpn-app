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
import MullvadTypes

final class WelcomeInteractor: @unchecked Sendable {
    private let tunnelManager: TunnelManager

    /// Interval used for periodic polling account updates.
    private let accountUpdateTimerInterval: Duration = .seconds(15)
    private var accountUpdateTimer: DispatchSourceTimer?

    private let logger = Logger(label: "\(WelcomeInteractor.self)")
    private var tunnelObserver: TunnelObserver?

    private lazy var accountDataPoller: AccountDataPoller = {
        AccountDataPoller(logger: logger, tunnelManager: tunnelManager)
    }()

    var didAddMoreCredit: (() -> Void)?

    var viewWillAppear = false {
        didSet {
            guard viewWillAppear else { return }
            accountDataPoller.startAccountUpdateTimer()
        }
    }

    var viewDidDisappear = false {
        didSet {
            guard viewDidDisappear else { return }
            accountDataPoller.stopAccountUpdateTimer()
        }
    }

    var accountNumber: String {
        tunnelManager.deviceState.accountData?.number ?? ""
    }

    var viewModel: WelcomeViewModel {
        WelcomeViewModel(
            deviceName: tunnelManager.deviceState.deviceData?.capitalizedName ?? "",
            accountNumber: tunnelManager.deviceState.accountData?.number.formattedAccountNumber ?? ""
        )
    }

    init(
        tunnelManager: TunnelManager
    ) {
        self.tunnelManager = tunnelManager
        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                let isInactive = previousDeviceState.accountData?.isExpired == true
                let isActive = deviceState.accountData?.isExpired == false
                if isInactive && isActive {
                    self?.didAddMoreCredit?()
                }
            })

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver
    }
}
