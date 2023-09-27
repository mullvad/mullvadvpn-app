//
//  Actor+ConnectionMonitoring.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    /// Assign a closure receiving tunnel monitor events.
    func setTunnelMonitorEventHandler() {
        tunnelMonitor.onEvent = { [weak self] event in
            /// Dispatch tunnel monitor events via command channel to guarantee the order of execution.
            self?.commandChannel.send(.monitorEvent(event))
        }
    }

    /**
     Handle tunnel monitor event.

     Invoked by comand consumer.

     - Important: this method will suspend and must only be invoked as a part of channel consumer to guarantee transactional execution.
     */
    func handleMonitorEvent(_ event: TunnelMonitorEvent) async {
        switch event {
        case .connectionEstablished:
            onEstablishConnection()

        case .connectionLost:
            await onHandleConnectionRecovery()
        }
    }

    /// Reset connection attempt counter and update actor state to `connected` state once connection is established.
    private func onEstablishConnection() {
        switch state {
        case var .connecting(connState), var .reconnecting(connState):
            // Reset connection attempt once successfully connected.
            connState.connectionAttemptCount = 0
            state = .connected(connState)

        case .initial, .connected, .disconnecting, .disconnected, .error:
            break
        }
    }

    /// Increment connection attempt counter and reconnect the tunnel.
    private func onHandleConnectionRecovery() async {
        func mutateConnectionState(_ connState: inout ConnectionState) {
            // Increment attempt counter
            connState.incrementAttemptCount()
        }

        switch state {
        case var .connecting(connState):
            mutateConnectionState(&connState)
            state = .connecting(connState)

        case var .reconnecting(connState):
            mutateConnectionState(&connState)
            state = .reconnecting(connState)

        case var .connected(connState):
            mutateConnectionState(&connState)
            state = .connected(connState)

        case .initial, .disconnected, .disconnecting, .error:
            // Explcit return to prevent reconnecting the tunnel.
            return
        }

        // Tunnel monitor should already be paused at this point so don't stop it to avoid a reset of its internal
        // counters.
        commandChannel.send(.reconnect(.random, stopTunnelMonitor: false))
    }
}
