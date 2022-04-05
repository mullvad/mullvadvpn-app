//
//  StopTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class StopTunnelOperation: ResultOperation<(), TunnelManager.Error> {
    private let queue: DispatchQueue
    private let state: TunnelManager.State

    init(queue: DispatchQueue, state: TunnelManager.State, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state

        super.init(completionQueue: queue, completionHandler: completionHandler)
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.finish(completion: .cancelled)
                return
            }

            guard let tunnel = self.state.tunnel else {
                self.finish(completion: .failure(.unsetAccount))
                return
            }

            switch self.state.tunnelStatus.state {
            case .disconnecting(.reconnect):
                self.state.tunnelStatus.state = .disconnecting(.nothing)

                self.finish(completion: .success(()))

            case .connected, .connecting, .reconnecting:
                // Disable on-demand when stopping the tunnel to prevent it from coming back up
                tunnel.isOnDemandEnabled = false

                tunnel.saveToPreferences { error in
                    self.queue.async {
                        if let error = error {
                            self.finish(completion: .failure(.saveVPNConfiguration(error)))
                        } else {
                            tunnel.stop()
                            self.finish(completion: .success(()))
                        }
                    }
                }

            default:
                self.finish(completion: .success(()))
            }
        }
    }
}
