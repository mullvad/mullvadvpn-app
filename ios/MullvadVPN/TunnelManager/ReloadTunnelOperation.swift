//
//  ReloadTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 10/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

protocol ReloadTunnelOperationDelegate: AnyObject {
    func operationDidRequestTunnelProvider(_ operation: Operation) -> TunnelProviderManagerType?
    func operation(_ operation: Operation, didFinishReloadingTunnelWithCompletion: OperationCompletion<(), TunnelManager.Error>)
}

class ReloadTunnelOperation: BaseTunnelOperation<(), TunnelManager.Error> {
    private weak var delegate: ReloadTunnelOperationDelegate?
    private var statusObserver: NSObjectProtocol?

    init(queue: DispatchQueue, delegate: ReloadTunnelOperationDelegate) {
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
                self.completeOperation(completion: .cancelled)
            }
        }
    }

    private func handleStatus(_ status: NEVPNStatus, ipcSession: TunnelIPC.Session) {
        guard isCancelled else {
            completeOperation(completion: .cancelled)
            return
        }

        switch status {
        case .connected:
            removeStatusObserver()

            ipcSession.reloadTunnelSettings { [weak self] error in
                guard let self = self else { return }

                self.queue.async {
                    self.completeOperation(completion: error.map { .failure(.reloadTunnel($0)) } ?? .success(()))
                }
            }

        case .connecting, .reasserting:
            // wait for transition to complete
            break

        case .invalid, .disconnecting, .disconnected:
            removeStatusObserver()
            completeOperation(completion: .success(()))

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

    override func completeOperation(completion: OperationCompletion<(), TunnelManager.Error>) {
        delegate?.operation(self, didFinishReloadingTunnelWithCompletion: completion)
        delegate = nil

        super.completeOperation(completion: completion)
    }

}
