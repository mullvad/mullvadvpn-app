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
        Task {
            switch await interactor.getTunnelStatus().state {
            case .disconnecting(.reconnect):
                await interactor.updateTunnelStatus { tunnelStatus in
                    tunnelStatus.state = .disconnecting(.nothing)
                }
                finish(result: .success(()))

            case .connected, .connecting, .reconnecting, .waitingForConnectivity(.noConnection), .error,
                .negotiatingEphemeralPeer:
                await doShutDownTunnel()

            case .disconnected, .disconnecting, .pendingReconnect, .waitingForConnectivity(.noNetwork):
                finish(result: .success(()))
            }
        }
    }

    private func doShutDownTunnel() async {
        guard let tunnel = await interactor.getTunnel() else {
            finish(result: .failure(UnsetTunnelError()))
            return
        }

        await tunnel.setOnDemandEnabled(enabled: isOnDemandEnabled)

        let error = await tunnel.saveToPreferences()
        if let error {
            finish(result: .failure(error))
        } else {
            await tunnel.stop()
            finish(result: .success(()))
        }
    }
}
