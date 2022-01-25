//
//  StopTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol StopTunnelOperationDelegate: AnyObject {
    func operationDidRequestTunnelState(_ operation: Operation) -> TunnelState
    func operation(_ operation: Operation, didSetTunnelState newTunnelState: TunnelState)
    func operationDidRequestTunnelProvider(_ operation: Operation) -> TunnelProviderManagerType?
    func operation(_ operation: Operation, didStopTunnelWithCompletion completion: OperationCompletion<(), TunnelManager.Error>)
}

class StopTunnelOperation: BaseTunnelOperation<(), TunnelManager.Error> {
    private weak var delegate: StopTunnelOperationDelegate?

    init(queue: DispatchQueue, delegate: StopTunnelOperationDelegate) {
        self.delegate = delegate
        super.init(queue: queue)
    }

    override func main() {
        queue.async {
            guard !self.isCancelled else {
                self.completeOperation(completion: .cancelled)
                return
            }

            guard let tunnelProvider = self.delegate?.operationDidRequestTunnelProvider(self) else {
                self.completeOperation(completion: .failure(.missingAccount))
                return
            }

            let tunnelState = self.delegate?.operationDidRequestTunnelState(self)

            switch tunnelState {
            case .disconnecting(.reconnect):
                self.delegate?.operation(self, didSetTunnelState: .disconnecting(.nothing))
                self.completeOperation(completion: .success(()))

            case .connected, .connecting:
                // Disable on-demand when stopping the tunnel to prevent it from coming back up
                tunnelProvider.isOnDemandEnabled = false

                tunnelProvider.saveToPreferences { error in
                    self.queue.async {
                        if let error = error {
                            self.completeOperation(completion: .failure(.saveVPNConfiguration(error)))
                        } else {
                            tunnelProvider.connection.stopVPNTunnel()
                            self.completeOperation(completion: .success(()))
                        }
                    }
                }

            default:
                self.completeOperation(completion: .success(()))
            }
        }
    }

    override func completeOperation(completion: OperationCompletion<(), TunnelManager.Error>) {
        delegate?.operation(self, didStopTunnelWithCompletion: completion)
        delegate = nil

        super.completeOperation(completion: completion)
    }
}
