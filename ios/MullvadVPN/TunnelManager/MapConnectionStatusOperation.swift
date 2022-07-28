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

    private let state: TunnelManager.State
    private let connectionStatus: NEVPNStatus
    private var startTunnelHandler: StartTunnelHandler?
    private var request: Cancellable?

    private let logger = Logger(label: "TunnelManager.MapConnectionStatusOperation")

    init(
        queue: DispatchQueue,
        state: TunnelManager.State,
        connectionStatus: NEVPNStatus,
        startTunnelHandler: @escaping StartTunnelHandler
    )
    {
        self.state = state
        self.connectionStatus = connectionStatus
        self.startTunnelHandler = startTunnelHandler

        super.init(dispatchQueue: queue)
    }

    override func main() {
        guard let tunnel = state.tunnel else {
            finish()
            return
        }

        let tunnelState = state.tunnelStatus.state

        switch connectionStatus {
        case .connecting:
            switch tunnelState {
            case .connecting(.some(_)):
                break
            default:
                state.tunnelStatus.state = .connecting(nil)
            }

            updateTunnelRelayAndFinish(tunnel: tunnel) { relay in
                return relay.map { .connecting($0) }
            }
            return

        case .reasserting:
            updateTunnelRelayAndFinish(tunnel: tunnel) { relay in
                return relay.map { .reconnecting($0) }
            }
            return

        case .connected:
            updateTunnelRelayAndFinish(tunnel: tunnel) { relay in
                return relay.map { .connected($0) }
            }
            return

        case .disconnected:
            switch tunnelState {
            case .pendingReconnect:
                logger.debug("Ignore disconnected state when pending reconnect.")

            case .disconnecting(.reconnect):
                logger.debug("Restart the tunnel on disconnect.")

                state.tunnelStatus.reset(to: .pendingReconnect)

                startTunnelHandler?()
                startTunnelHandler = nil

            default:
                state.tunnelStatus.reset(to: .disconnected)
            }

        case .disconnecting:
            switch tunnelState {
            case .disconnecting:
                break
            default:
                state.tunnelStatus.reset(to: .disconnecting(.nothing))
            }

        case .invalid:
            state.tunnelStatus.reset(to: .disconnected)

        @unknown default:
            logger.debug("Unknown NEVPNStatus: \(connectionStatus.rawValue)")
        }

        finish()
    }

    override func operationDidCancel() {
        request?.cancel()
    }

    private func updateTunnelRelayAndFinish(
        tunnel: Tunnel,
        mapRelayToState: @escaping (PacketTunnelRelay?) -> TunnelState?
    )
    {
        request = tunnel.getTunnelStatus { [weak self] completion in
            guard let self = self else { return }

            self.dispatchQueue.async {
                if case .success(let packetTunnelStatus) = completion, !self.isCancelled {
                    self.state.tunnelStatus.update(
                        from: packetTunnelStatus,
                        mappingRelayToState: mapRelayToState
                    )
                }

                self.finish()
            }
        }
    }
}
