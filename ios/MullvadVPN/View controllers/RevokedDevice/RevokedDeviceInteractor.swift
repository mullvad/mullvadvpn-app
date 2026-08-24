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

protocol RevokedDeviceInteractorProtocol {
    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)? { get set }
    var tunnelStatus: TunnelStatus { get }
}

final class RevokedDeviceInteractor: RevokedDeviceInteractorProtocol {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?

    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)?

    var tunnelStatus: TunnelStatus {
        tunnelManager.tunnelStatus
    }

    init(tunnelManager: TunnelManager) {
        self.tunnelManager = tunnelManager

        let tunnelObserver =
            TunnelBlockObserver(didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                self?.didUpdateTunnelStatus?(tunnelStatus)
            })

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }
}

class MockRevokedDeviceInteractor: RevokedDeviceInteractorProtocol {
    var tunnelStatus: TunnelStatus
    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)?

    init(tunnelStatus: TunnelStatus) {
        self.tunnelStatus = tunnelStatus
    }
}
