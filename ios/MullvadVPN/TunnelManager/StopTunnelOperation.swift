//
//  StopTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class StopTunnelOperation: ResultOperation<(), TunnelManager.Error> {
    private let state: TunnelManager.State

    init(
        dispatchQueue: DispatchQueue,
        state: TunnelManager.State,
        completionHandler: @escaping CompletionHandler
    )
    {
        self.state = state

        super.init(
            dispatchQueue: dispatchQueue,
            completionQueue: dispatchQueue,
            completionHandler: completionHandler
        )
    }

    override func main() {
        guard let tunnel = state.tunnel else {
            finish(completion: .failure(.unsetAccount))
            return
        }

        switch state.tunnelStatus.state {
        case .disconnecting(.reconnect):
            state.tunnelStatus.state = .disconnecting(.nothing)

            finish(completion: .success(()))

        case .connected, .connecting, .reconnecting:
            // Disable on-demand when stopping the tunnel to prevent it from coming back up
            tunnel.isOnDemandEnabled = false

            tunnel.saveToPreferences { error in
                self.dispatchQueue.async {
                    if let error = error {
                        self.finish(completion: .failure(.saveVPNConfiguration(error)))
                    } else {
                        tunnel.stop()
                        self.finish(completion: .success(()))
                    }
                }
            }

        default:
            finish(completion: .success(()))
        }
    }
}
