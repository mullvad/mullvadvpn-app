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
import Operations

class StopTunnelOperation: ResultOperation<Void>, @unchecked Sendable {
    private let interactor: TunnelInteractor
    var isOnDemandEnabled = false

    init(
        dispatchQueue: DispatchQueue,
        interactor: TunnelInteractor,
        completionHandler: @escaping CompletionHandler
    ) {
        self.interactor = interactor

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        switch interactor.tunnelStatus.state {
        case .disconnecting(.reconnect):
            interactor.updateTunnelStatus { tunnelStatus in
                tunnelStatus.state = .disconnecting(.nothing)
            }

            finish(result: .success(()))

        case .connected, .connecting, .reconnecting, .waitingForConnectivity(.noConnection), .error,
            .negotiatingEphemeralPeer:
            doShutDownTunnel()

        case .disconnected, .disconnecting, .pendingReconnect, .waitingForConnectivity(.noNetwork):
            finish(result: .success(()))
        }
    }

    private func doShutDownTunnel() {
        guard let tunnel = interactor.tunnel else {
            finish(result: .failure(UnsetTunnelError()))
            return
        }

        tunnel.isOnDemandEnabled = isOnDemandEnabled

        tunnel.saveToPreferences { error in
            self.dispatchQueue.async {
                if let error {
                    self.finish(result: .failure(error))
                } else {
                    tunnel.stop()
                    self.finish(result: .success(()))
                }
            }
        }
    }
}
