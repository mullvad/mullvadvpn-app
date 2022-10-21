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
import TunnelProviderMessaging

class MapConnectionStatusOperation: AsyncOperation {
    private let interactor: TunnelInteractor
    private let connectionStatus: NEVPNStatus
    private var request: Cancellable?

    private let logger = Logger(label: "TunnelManager.MapConnectionStatusOperation")

    init(
        queue: DispatchQueue,
        interactor: TunnelInteractor,
        connectionStatus: NEVPNStatus
    ) {
        self.interactor = interactor
        self.connectionStatus = connectionStatus

        super.init(dispatchQueue: queue)
    }

    override func main() {
        guard let tunnel = interactor.tunnel else {
            finish()
            return
        }

        let tunnelState = interactor.tunnelStatus.state

        switch connectionStatus {
        case .connecting:
            switch tunnelState {
            case .connecting:
                break

            default:
                interactor.updateTunnelStatus { tunnelStatus in
                    tunnelStatus.state = .connecting(nil)
                }
            }

            fetchTunnelStatus(tunnel: tunnel) { packetTunnelStatus in
                if packetTunnelStatus.isNetworkReachable {
                    return packetTunnelStatus.tunnelRelay.map { .connecting($0) }
                } else {
                    return .waitingForConnectivity
                }
            }
            return

        case .reasserting:
            fetchTunnelStatus(tunnel: tunnel) { packetTunnelStatus in
                if packetTunnelStatus.isNetworkReachable {
                    return packetTunnelStatus.tunnelRelay.map { .reconnecting($0) }
                } else {
                    return .waitingForConnectivity
                }
            }
            return

        case .connected:
            fetchTunnelStatus(tunnel: tunnel) { packetTunnelStatus in
                if packetTunnelStatus.isNetworkReachable {
                    return packetTunnelStatus.tunnelRelay.map { .connected($0) }
                } else {
                    return .waitingForConnectivity
                }
            }
            return

        case .disconnected:
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
                interactor.updateTunnelStatus { tunnelStatus in
                    tunnelStatus = TunnelStatus()
                    tunnelStatus.state = .disconnected
                }
            }

        case .disconnecting:
            switch tunnelState {
            case .disconnecting:
                break
            default:
                interactor.updateTunnelStatus { tunnelStatus in
                    tunnelStatus = TunnelStatus()
                    tunnelStatus.state = .disconnecting(.nothing)
                }
            }

        case .invalid:
            interactor.updateTunnelStatus { tunnelStatus in
                tunnelStatus = TunnelStatus()
                tunnelStatus.state = .disconnected
            }

        @unknown default:
            logger.debug("Unknown NEVPNStatus: \(connectionStatus.rawValue)")
        }

        finish()
    }

    override func operationDidCancel() {
        request?.cancel()
    }

    private func fetchTunnelStatus(
        tunnel: Tunnel,
        mapToState: @escaping (PacketTunnelStatus) -> TunnelState?
    ) {
        request = tunnel.getTunnelStatus { [weak self] completion in
            guard let self = self else { return }

            self.dispatchQueue.async {
                if case let .success(packetTunnelStatus) = completion, !self.isCancelled {
                    self.interactor.updateTunnelStatus { tunnelStatus in
                        tunnelStatus.packetTunnelStatus = packetTunnelStatus

                        if let newState = mapToState(packetTunnelStatus) {
                            tunnelStatus.state = newState
                        }
                    }
                }

                self.finish()
            }
        }
    }
}
