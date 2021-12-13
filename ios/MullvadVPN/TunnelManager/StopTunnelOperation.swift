//
//  StopTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol StopTunnelOperationDelegate {
    func operationDidRequestTunnelState(_ operation: Operation) -> TunnelState
    func operation(_ operation: Operation, didSetTunnelState newTunnelState: TunnelState)
    func operationDidRequestTunnelProvider(_ operation: Operation) -> TunnelProviderManagerType?
}

class StopTunnelOperation: AsyncOperation {
    typealias CompletionHandler = (TunnelManager.Error?) -> Void

    private let queue: DispatchQueue
    private let delegate: StopTunnelOperationDelegate
    private var completionHandler: CompletionHandler?

    init(queue: DispatchQueue, delegate: StopTunnelOperationDelegate, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.delegate = delegate
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.finish(error: nil)
                return
            }

            guard let tunnelProvider = self.delegate.operationDidRequestTunnelProvider(self) else {
                self.finish(error: .missingAccount)
                return
            }

            let tunnelState = self.delegate.operationDidRequestTunnelState(self)

            switch tunnelState {
            case .disconnecting(.reconnect):
                self.delegate.operation(self, didSetTunnelState: .disconnecting(.nothing))
                self.finish(error: nil)

            case .connected, .connecting:
                // Disable on-demand when stopping the tunnel to prevent it from coming back up
                tunnelProvider.isOnDemandEnabled = false

                tunnelProvider.saveToPreferences { error in
                    self.queue.async {
                        if let error = error {
                            self.finish(error: .saveVPNConfiguration(error))
                        } else {
                            tunnelProvider.connection.stopVPNTunnel()
                            self.finish(error: nil)
                        }
                    }
                }

            default:
                self.finish(error: nil)
            }
        }
    }

    private func finish(error: TunnelManager.Error?) {
        completionHandler?(error)
        completionHandler = nil

        finish()
    }
}
