//
//  ReloadTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 10/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import NetworkExtension

protocol ReloadTunnelOperationDelegate {
    func operationDidRequestTunnelProvider(_ operation: Operation) -> TunnelProviderManagerType?
}

class ReloadTunnelOperation: AsyncOperation {
    private let queue: DispatchQueue
    private let delegate: ReloadTunnelOperationDelegate
    private var statusObserver: NSObjectProtocol?

    private let logger = Logger(label: "TunnelManager.ReloadTunnelOperation")

    init(queue: DispatchQueue, delegate: ReloadTunnelOperationDelegate) {
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
                self.finish()
                return
            }

            let ipcSession = TunnelIPC.Session(from: tunnelProvider)

            // Add observer
            self.statusObserver = NotificationCenter.default.addObserver(
                forName: .NEVPNStatusDidChange,
                object: tunnelProvider.connection,
                queue: nil) { [weak self] notification in
                    guard let self = self else { return }
                    guard let connection = notification.object as? VPNConnectionProtocol else { return }

                    self.queue.async {
                        self.handleStatus(connection.status, ipcSession: ipcSession)
                    }
                }

            // Run initial check
            self.handleStatus(tunnelProvider.connection.status, ipcSession: ipcSession)
        }
    }

    override func cancel() {
        super.cancel()

        queue.async {
            self.removeStatusObserver()

            if self.isExecuting {
                self.finish()
            }
        }
    }

    private func handleStatus(_ status: NEVPNStatus, ipcSession: TunnelIPC.Session) {
        guard isCancelled else {
            finish()
            return
        }

        switch status {
        case .connected:
            removeStatusObserver()

            ipcSession.reloadTunnelSettings { [weak self] error in
                guard let self = self else { return }

                if let error = error {
                    self.logger.error(chainedError: error, message: "Failed to send IPC request to reload tunnel settings")
                }

                self.finish()
            }

        case .connecting, .reasserting:
            // wait for transition to complete
            break

        case .invalid, .disconnecting, .disconnected:
            removeStatusObserver()
            finish()

        @unknown default:
            break
        }
    }

    private func removeStatusObserver() {
        if let statusObserver = statusObserver {
            NotificationCenter.default.removeObserver(statusObserver)

            self.statusObserver = nil
        }
    }

}
