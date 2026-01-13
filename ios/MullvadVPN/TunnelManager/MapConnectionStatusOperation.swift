//
//  MapConnectionStatusOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 15/12/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import NetworkExtension
import Operations
import PacketTunnelCore

class MapConnectionStatusOperation: AsyncOperation, @unchecked Sendable {
    private let interactor: TunnelInteractor
    private let connectionStatus: NEVPNStatus
    private var request: Cancellable?
    private var pathStatus: Network.NWPath.Status?

    private let logger = Logger(label: "TunnelManager.MapConnectionStatusOperation")

    required init(
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
                    return .connected(
                        connectionState.selectedRelays,
                        isPostQuantum: connectionState.isPostQuantum,
                        isDaita: connectionState.isDaitaEnabled
                    )
                case let .connecting(connectionState):
                    return .connecting(
                        connectionState.selectedRelays,
                        isPostQuantum: connectionState.isPostQuantum,
                        isDaita: connectionState.isDaitaEnabled
                    )
                case let .negotiatingEphemeralPeer(connectionState, privateKey):
                    return .negotiatingEphemeralPeer(
                        connectionState.selectedRelays,
                        privateKey,
                        isPostQuantum: connectionState.isPostQuantum,
                        isDaita: connectionState.isDaitaEnabled
                    )
                case let .reconnecting(connectionState):
                    return .reconnecting(
                        connectionState.selectedRelays,
                        isPostQuantum: connectionState.isPostQuantum,
                        isDaita: connectionState.isDaitaEnabled
                    )
                case let .error(blockedState):
                    return .error(blockedState.reason)
                case .initial, .disconnecting, .disconnected:
                    return .none
                }
            }
            return

        case .disconnected:
            if handleDisconnectedState(tunnelState) {
                // Async IPC verification started, finish() will be called by the callback
                return
            }

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
        // Intentionally don't cancel the IPC request. Let it complete so we can use its result.
        // With aggressive polling, new operations may cancel previous ones before IPC completes.
        // By allowing the IPC to finish, we ensure status updates aren't lost.
        // The MutuallyExclusive constraint ensures operations complete in order.
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
                    tunnelStatus.state =
                        isNetworkReachable
                        ? .disconnecting(.nothing)
                        : .waitingForConnectivity(.noNetwork)
                }
            }
        }
    }

    /// Handles the disconnected NEVPNStatus.
    /// - Returns: `true` if an async IPC verification was started (caller should not call `finish()`),
    ///            `false` if handled synchronously (caller should call `finish()`).
    private func handleDisconnectedState(_ tunnelState: TunnelState) -> Bool {
        switch tunnelState {
        case .pendingReconnect:
            logger.debug("Ignore disconnected state when pending reconnect.")
            return false

        case .disconnecting(.reconnect):
            logger.debug("Restart the tunnel on disconnect.")
            interactor.updateTunnelStatus { tunnelStatus in
                tunnelStatus = TunnelStatus()
                tunnelStatus.state = .pendingReconnect
            }
            interactor.startTunnel()
            return false

        case .connecting, .reconnecting, .negotiatingEphemeralPeer:
            // NEVPNStatus reports disconnected but UI expects a connecting state.
            // This could be transient (tunnel still starting) or real (tunnel crashed).
            // Attempt IPC to verify - if tunnel responds, it's alive and we use its state.
            // If IPC fails, the tunnel is dead and we accept the disconnect.
            guard let tunnel = interactor.tunnel else {
                setTunnelDisconnectedStatus()
                return false
            }

            logger.debug("Received disconnected while \(tunnelState) - verifying tunnel status via IPC.")
            verifyTunnelStatusOrDisconnect(tunnel: tunnel)
            return true  // Async operation started

        default:
            setTunnelDisconnectedStatus()
            return false
        }
    }

    /// Attempts to fetch tunnel status via IPC to verify if the tunnel is still alive.
    /// If IPC succeeds, updates the UI with the actual tunnel state.
    /// If IPC fails (tunnel crashed/unresponsive), sets the tunnel status to disconnected.
    private func verifyTunnelStatusOrDisconnect(tunnel: any TunnelProtocol) {
        request = tunnel.getTunnelStatus { [weak self] result in
            guard let self else { return }

            dispatchQueue.async {
                // Process results even if cancelled - the IPC result is still valid.
                // Cancellation just means a newer operation started.
                switch result {
                case let .success(observedState):
                    self.logger.debug("IPC succeeded - tunnel is alive, state: \(observedState.name).")
                    self.interactor.updateTunnelStatus { tunnelStatus in
                        tunnelStatus.observedState = observedState

                        // Map the observed state to tunnel state
                        if let newState = self.mapObservedStateToTunnelState(observedState) {
                            tunnelStatus.state = newState
                        }
                    }

                case let .failure(error):
                    // IPC failed - tunnel is dead or unresponsive
                    self.logger.debug("IPC failed (\(error.localizedDescription)) - tunnel is dead, accepting disconnect.")
                    self.setTunnelDisconnectedStatus()
                }

                self.finish()
            }
        }
    }

    /// Maps an ObservedState from the packet tunnel to a TunnelState for the UI.
    private func mapObservedStateToTunnelState(_ observedState: ObservedState) -> TunnelState? {
        switch observedState {
        case let .connected(connectionState):
            return .connected(
                connectionState.selectedRelays,
                isPostQuantum: connectionState.isPostQuantum,
                isDaita: connectionState.isDaitaEnabled
            )
        case let .connecting(connectionState):
            return .connecting(
                connectionState.selectedRelays,
                isPostQuantum: connectionState.isPostQuantum,
                isDaita: connectionState.isDaitaEnabled
            )
        case let .negotiatingEphemeralPeer(connectionState, privateKey):
            return .negotiatingEphemeralPeer(
                connectionState.selectedRelays,
                privateKey,
                isPostQuantum: connectionState.isPostQuantum,
                isDaita: connectionState.isDaitaEnabled
            )
        case let .reconnecting(connectionState):
            return .reconnecting(
                connectionState.selectedRelays,
                isPostQuantum: connectionState.isPostQuantum,
                isDaita: connectionState.isDaitaEnabled
            )
        case let .error(blockedState):
            return .error(blockedState.reason)
        case .initial, .disconnecting, .disconnected:
            return .disconnected
        }
    }

    private func setTunnelDisconnectedStatus() {
        interactor.updateTunnelStatus { tunnelStatus in
            tunnelStatus = TunnelStatus()
            tunnelStatus.state =
                pathStatus == .unsatisfied
                ? .waitingForConnectivity(.noNetwork)
                : .disconnected
        }
    }

    private func fetchTunnelStatus(
        tunnel: any TunnelProtocol,
        mapToState: @escaping @Sendable (ObservedState) -> TunnelState?
    ) {
        request = tunnel.getTunnelStatus { [weak self] result in
            guard let self else { return }

            dispatchQueue.async {
                // Process successful IPC results even if operation was cancelled.
                // Cancellation means a newer poll started, not that this result is invalid.
                // The MutuallyExclusive constraint ensures updates happen in order.
                if case let .success(observedState) = result {
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
