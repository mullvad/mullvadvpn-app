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

class MapConnectionStatusOperation: AsyncOperation {
    typealias StartTunnelHandler = () -> Void

    private let queue: DispatchQueue
    private let state: TunnelManager.State
    private let connectionStatus: NEVPNStatus
    private var startTunnelHandler: StartTunnelHandler?

    private let logger = Logger(label: "TunnelManager.MapConnectionStatusOperation")

    init(queue: DispatchQueue, state: TunnelManager.State, connectionStatus: NEVPNStatus, startTunnelHandler: @escaping StartTunnelHandler) {
        self.queue = queue
        self.state = state
        self.connectionStatus = connectionStatus
        self.startTunnelHandler = startTunnelHandler
    }

    override func main() {
        queue.async {
            self.execute()
        }
    }

    override func cancel() {
        super.cancel()

        queue.async {
            // Finish immediately if cancelled during execution.
            if self.isExecuting {
                self.finish()
            }
        }
    }

    private func execute() {
        guard let tunnelProvider = state.tunnelProvider, !isCancelled else {
            finish()
            return
        }

        let tunnelState = state.tunnelState

        switch connectionStatus {
        case .connecting:
            switch tunnelState {
            case .connecting(.some(_)):
                logger.debug("Ignore repeating connecting state.")
            default:
                state.tunnelState = .connecting(nil)
            }

        case .reasserting:
            requestTunnelRelay(from: tunnelProvider) { [weak self] result in
                guard let self = self else { return }

                if case .success(.some(let connectionInfo)) = result, !self.isCancelled {
                    self.state.tunnelState = .reconnecting(connectionInfo)
                }

                self.finish()
            }

            return

        case .connected:
            requestTunnelRelay(from: tunnelProvider) { [weak self] result in
                guard let self = self else { return }

                if case .success(.some(let connectionInfo)) = result, !self.isCancelled {
                    self.state.tunnelState = .connected(connectionInfo)
                }

                self.finish()
            }

            return

        case .disconnected:
            switch tunnelState {
            case .pendingReconnect:
                logger.debug("Ignore disconnected state when pending reconnect.")

            case .disconnecting(.reconnect):
                logger.debug("Restart the tunnel on disconnect.")

                state.tunnelState = .pendingReconnect

                startTunnelHandler?()
                startTunnelHandler = nil

            default:
                state.tunnelState = .disconnected
            }

        case .disconnecting:
            switch tunnelState {
            case .disconnecting:
                break
            default:
                state.tunnelState = .disconnecting(.nothing)
            }

        case .invalid:
            state.tunnelState = .disconnected

        @unknown default:
            logger.debug("Unknown NEVPNStatus: \(connectionStatus.rawValue)")
        }

        finish()
    }

    private func requestTunnelRelay(from tunnelProvider: TunnelProviderManagerType, completionHandler: @escaping (Result<TunnelConnectionInfo?, TunnelIPC.Error>?) -> Void) {
        guard tunnelProvider.connection.status == .reasserting || tunnelProvider.connection.status == .connected else {
            completionHandler(nil)
            return
        }

        let ipcSession = TunnelIPC.Session(from: tunnelProvider)

        ipcSession.getTunnelConnectionInfo { [weak self] result in
            guard let self = self else { return }

            self.queue.async {
                completionHandler(result)
            }
        }
    }
}
