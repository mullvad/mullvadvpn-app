//
//  MapConnectionStatusOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import Logging

protocol MapConnectionStatusOperationDelegate: AnyObject {
    func operationDidRequestTunnelState(_ operation: Operation) -> TunnelState
    func operation(_ operation: Operation, didSetTunnelState newTunnelState: TunnelState)
    func operationDidRequestTunnelProvider(_ operation: Operation) -> TunnelProviderManagerType?
    func operationDidRequestTunnelToStart(_ operation: Operation)
}

class MapConnectionStatusOperation: AsyncOperation {
    private let queue: DispatchQueue
    private let connectionStatus: NEVPNStatus
    private weak var delegate: MapConnectionStatusOperationDelegate?

    private let logger = Logger(label: "TunnelManager.UpdateTunnelStateOperation")

    init(queue: DispatchQueue, connectionStatus: NEVPNStatus, delegate: MapConnectionStatusOperationDelegate) {
        self.queue = queue
        self.connectionStatus = connectionStatus
        self.delegate = delegate
    }

    override func main() {
        queue.async {
            self.execute {
                self.finish()
            }
        }
    }

    private func execute(completionHandler: @escaping () -> Void) {
        guard !isCancelled else {
            completionHandler()
            return
        }

        guard let tunnelProvider = delegate?.operationDidRequestTunnelProvider(self),
              let tunnelState = delegate?.operationDidRequestTunnelState(self) else {
                  completionHandler()
                  return
              }

        switch connectionStatus {
        case .connecting:
            switch tunnelState {
            case .connecting(.some(_)):
                logger.debug("Ignore repeating connecting state.")
            default:
                delegate?.operation(self, didSetTunnelState: .connecting(nil))
            }
            completionHandler()

        case .reasserting:
            let ipcSession = TunnelIPC.Session(from: tunnelProvider)

            ipcSession.getTunnelConnectionInfo { result in
                self.queue.async {
                    if case .success(.some(let connectionInfo)) = result, !self.isCancelled {
                        self.delegate?.operation(self, didSetTunnelState: .reconnecting(connectionInfo))
                    }
                    completionHandler()
                }
            }

        case .connected:
            let ipcSession = TunnelIPC.Session(from: tunnelProvider)

            ipcSession.getTunnelConnectionInfo { result in
                self.queue.async {
                    if case .success(.some(let connectionInfo)) = result, !self.isCancelled {
                        self.delegate?.operation(self, didSetTunnelState: .connected(connectionInfo))
                    }
                    completionHandler()
                }
            }

        case .disconnected:
            switch tunnelState {
            case .pendingReconnect:
                logger.debug("Ignore disconnected state when pending reconnect.")

            case .disconnecting(.reconnect):
                logger.debug("Restart the tunnel on disconnect.")
                delegate?.operation(self, didSetTunnelState: .pendingReconnect)
                delegate?.operationDidRequestTunnelToStart(self)

            default:
                delegate?.operation(self, didSetTunnelState: .disconnected)
            }
            completionHandler()

        case .disconnecting:
            switch tunnelState {
            case .disconnecting:
                break
            default:
                delegate?.operation(self, didSetTunnelState: .disconnecting(.nothing))
            }
            completionHandler()

        case .invalid:
            delegate?.operation(self, didSetTunnelState: .disconnected)
            completionHandler()

        @unknown default:
            logger.debug("Unknown NEVPNStatus: \(connectionStatus.rawValue)")
            completionHandler()
        }
    }
}
