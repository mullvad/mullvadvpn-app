//
//  MapConnectionStatusOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import NetworkExtension
import Operations
import PacketTunnelCore

class MapConnectionStatusOperation: AsyncOperation {
    private let interactor: TunnelInteractor
    private let connectionStatus: NEVPNStatus
    private var request: Cancellable?
    private var pathStatus: Network.NWPath.Status?

    private let logger = Logger(label: "TunnelManager.MapConnectionStatusOperation")

    init(
        queue: DispatchQueue,
        interactor: TunnelInteractor,
        connectionStatus: NEVPNStatus,
        networkStatus: Network.NWPath.Status?
    ) {
        self.interactor = interactor
        self.connectionStatus = connectionStatus
        pathStatus = networkStatus

        super.init(dispatchQueue: queue)
    }

    override func main() {
        guard let tunnel = interactor.tunnel else {
            setTunnelDisconnectedStatus()

            finish()
            return
        }

        let tunnelState = interactor.tunnelStatus.state

        switch connectionStatus {
        case .connecting, .reasserting, .connected:
            fetchTunnelStatus(tunnel: tunnel) { observedState in
                switch observedState {
                case let .connected(connectionState):
                    return connectionState.isNetworkReachable
                        ? .connected(connectionState.selectedRelay)
                        : .waitingForConnectivity(.noConnection)
                case let .connecting(connectionState):
                    return connectionState.isNetworkReachable
                        ? .connecting(connectionState.selectedRelay)
                        : .waitingForConnectivity(.noConnection)
                case let .reconnecting(connectionState):
                    return connectionState.isNetworkReachable
                        ? .reconnecting(connectionState.selectedRelay)
                        : .waitingForConnectivity(.noConnection)
                case let .error(blockedState):
                    return .error(blockedState.reason)
                case .initial, .disconnecting, .disconnected:
                    return .none
                }
            }
            return

        case .disconnected:
            handleDisconnectedState(tunnelState)

        case .disconnecting:
            handleDisconnectingState(tunnelState)

        case .invalid:
            setTunnelDisconnectedStatus()

        @unknown default:
            logger.debug("Unknown NEVPNStatus: \(connectionStatus.rawValue)")
        }

        finish()
    }

    override func operationDidCancel() {
        request?.cancel()
    }

    private func handleDisconnectingState(_ tunnelState: TunnelState) {
        switch tunnelState {
        case .disconnecting:
            break
        default:
            interactor.updateTunnelStatus { tunnelStatus in
                // Avoid displaying waiting for connectivity banners if the tunnel in a blocked state when disconnecting
                if tunnelStatus.observedState.blockedState != nil {
                    tunnelStatus.state = .disconnecting(.nothing)
                } else {
                    let isNetworkReachable = tunnelStatus.observedState.connectionState?.isNetworkReachable ?? false
                    tunnelStatus.state = isNetworkReachable
                        ? .disconnecting(.nothing)
                        : .waitingForConnectivity(.noNetwork)
                }
            }
        }
    }

    private func handleDisconnectedState(_ tunnelState: TunnelState) {
        switch tunnelState {
        case .pendingReconnect:
            logger.debug("Ignore disconnected state when pending reconnect.")

        case .disconnecting(.reconnect):
            logger.debug("Restart the tunnel on disconnect.")
            interactor.updateTunnelStatus { tunnelStatus in
                tunnelStatus = TunnelStatus()
                tunnelStatus.state = .pendingReconnect
            }
            interactor.startTunnel()

        default:
            setTunnelDisconnectedStatus()
        }
    }

    private func setTunnelDisconnectedStatus() {
        interactor.updateTunnelStatus { tunnelStatus in
            tunnelStatus = TunnelStatus()
            tunnelStatus.state = pathStatus == .unsatisfied
                ? .waitingForConnectivity(.noNetwork)
                : .disconnected
        }
    }

    private func fetchTunnelStatus(
        tunnel: any TunnelProtocol,
        mapToState: @escaping (ObservedState) -> TunnelState?
    ) {
        request = tunnel.getTunnelStatus { [weak self] result in
            guard let self else { return }

            dispatchQueue.async {
                if case let .success(observedState) = result, !self.isCancelled {
                    self.interactor.updateTunnelStatus { tunnelStatus in
                        tunnelStatus.observedState = observedState

                        if let newState = mapToState(observedState) {
                            tunnelStatus.state = newState
                        }
                    }
                }

                self.finish()
            }
        }
    }
}
