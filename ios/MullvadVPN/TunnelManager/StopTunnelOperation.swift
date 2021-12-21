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
    func operation(_ operation: Operation, didFailToStopTunnelWithError error: TunnelManager.Error)
}

class StopTunnelOperation: AsyncOperation {
    private let queue: DispatchQueue
    private let delegate: StopTunnelOperationDelegate

    init(queue: DispatchQueue, delegate: StopTunnelOperationDelegate) {
        self.queue = queue
        self.delegate = delegate
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.finish()
                return
            }

            guard let tunnelProvider = self.delegate.operationDidRequestTunnelProvider(self) else {
                self.delegate.operation(self, didFailToStopTunnelWithError: .missingAccount)
                self.finish()
                return
            }

            let tunnelState = self.delegate.operationDidRequestTunnelState(self)

            switch tunnelState {
            case .disconnecting(.reconnect):
                self.delegate.operation(self, didSetTunnelState: .disconnecting(.nothing))
                self.finish()

            case .connected, .connecting:
                // Disable on-demand when stopping the tunnel to prevent it from coming back up
                tunnelProvider.isOnDemandEnabled = false

                tunnelProvider.saveToPreferences { error in
                    self.queue.async {
                        if let error = error {
                            self.delegate.operation(self, didFailToStopTunnelWithError: .saveVPNConfiguration(error))
                            self.finish()
                        } else {
                            tunnelProvider.connection.stopVPNTunnel()
                            self.finish()
                        }
                    }
                }

            default:
                self.finish()
            }
        }
    }
}
