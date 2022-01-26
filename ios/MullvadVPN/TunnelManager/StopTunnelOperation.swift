//
//  StopTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class StopTunnelOperation: AsyncOperation {
    typealias CompletionHandler = (OperationCompletion<(), TunnelManager.Error>) -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private var completionHandler: CompletionHandler?

    init(queue: DispatchQueue, state: TunnelManager.State, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            self.execute { completion in
                self.completionHandler?(completion)
                self.completionHandler = nil

                self.finish()
            }
        }
    }

    private func execute(completionHandler: @escaping CompletionHandler) {
        guard !isCancelled else {
            completionHandler(.cancelled)
            return
        }

        guard let tunnelProvider = state.tunnelProvider else {
            completionHandler(.failure(.missingAccount))
            return
        }

        switch self.state.tunnelState {
        case .disconnecting(.reconnect):
            state.tunnelState = .disconnecting(.nothing)

            completionHandler(.success(()))

        case .connected, .connecting:
            // Disable on-demand when stopping the tunnel to prevent it from coming back up
            tunnelProvider.isOnDemandEnabled = false

            tunnelProvider.saveToPreferences { error in
                self.queue.async {
                    if let error = error {
                        completionHandler(.failure(.saveVPNConfiguration(error)))
                    } else {
                        tunnelProvider.connection.stopVPNTunnel()
                        completionHandler(.success(()))
                    }
                }
            }

        default:
            completionHandler(.success(()))
        }
    }
}
