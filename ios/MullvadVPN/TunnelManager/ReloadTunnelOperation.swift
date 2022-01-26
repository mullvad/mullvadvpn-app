//
//  ReloadTunnelOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 10/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

class ReloadTunnelOperation: AsyncOperation {
    typealias CompletionHandler = (OperationCompletion<(), TunnelManager.Error>) -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private var completionHandler: CompletionHandler?
    private var statusObserver: NSObjectProtocol?

    init(queue: DispatchQueue, state: TunnelManager.State, completionHandler: @escaping CompletionHandler) {
        self.queue = queue
        self.state = state
        self.completionHandler = completionHandler
    }

    override func main() {
        queue.async {
            self.execute { [weak self] completion in
                self?.completeOperation(completion: completion)
            }
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

    private func completeOperation(completion: OperationCompletion<(), TunnelManager.Error>) {
        completionHandler?(completion)
        completionHandler = nil

        finish()
    }

    private func execute(completionHandler: @escaping CompletionHandler) {
        guard !isCancelled else {
            completionHandler(.cancelled)
            return
        }

        guard let tunnelProvider = self.state.tunnelProvider else {
            completionHandler(.failure(.missingAccount))
            return
        }

        let ipcSession = TunnelIPC.Session(from: tunnelProvider)

        // Add observer
        statusObserver = NotificationCenter.default.addObserver(
            forName: .NEVPNStatusDidChange,
            object: tunnelProvider.connection,
            queue: nil) { [weak self] notification in
                guard let self = self else { return }
                guard let connection = notification.object as? VPNConnectionProtocol else { return }

                self.queue.async {
                    self.handleStatus(connection.status, ipcSession: ipcSession, completionHandler: completionHandler)
                }
            }

        // Run initial check
        handleStatus(tunnelProvider.connection.status, ipcSession: ipcSession, completionHandler: completionHandler)
    }

    private func handleStatus(_ status: NEVPNStatus, ipcSession: TunnelIPC.Session, completionHandler: @escaping CompletionHandler) {
        guard !isCancelled else {
            completionHandler(.cancelled)
            return
        }

        switch status {
        case .connected:
            removeStatusObserver()

            ipcSession.reloadTunnelSettings { [weak self] error in
                guard let self = self else { return }

                self.queue.async {
                    completionHandler(error.map { .failure(.reloadTunnel($0)) } ?? .success(()))
                }
            }

        case .connecting, .reasserting:
            // wait for transition to complete
            break

        case .invalid, .disconnecting, .disconnected:
            removeStatusObserver()
            completionHandler(.success(()))

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
