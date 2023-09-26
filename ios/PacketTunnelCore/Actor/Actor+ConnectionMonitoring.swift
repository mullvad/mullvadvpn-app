//
//  Actor+ConnectionMonitoring.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    /**
     Assign closure receiving tunnel monitor events.
      */
    func setTunnelMonitorEventHandler() {
        tunnelMonitor.onEvent = { [weak self] event in
            guard let self else { return }
            Task { await self.enqueueMonitorEvent(event) }
        }
    }

    /**
     Enqueue a task handling monitor event.

     - Important: Internal implementation must not call this method from operation executing on `TaskQueue`.
     */
    private func enqueueMonitorEvent(_ event: TunnelMonitorEvent) async {
        try? await taskQueue.add(kind: .monitorEvent) { [self] in
            switch event {
            case .connectionEstablished:
                onEstablishConnection()
            case .connectionLost:
                await onHandleConnectionRecovery()
            }
        }
    }

    /**
     Reset connection attempt counter and update actor state to `connected` state once connection is established.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue`.
     */
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

    /**
     Increment connection attempt counter and reconnect the tunnel.

     - Important: This method must only be invoked as a part of operation scheduled on `TaskQueue`.
     */
    private func onHandleConnectionRecovery() async {
        switch state {
        case var .connecting(connState), var .reconnecting(connState), var .connected(connState):
            guard let targetState = state.targetStateForReconnect else { return }

            // Increment attempt counter
            connState.incrementAttemptCount()

            switch targetState {
            case .connecting:
                state = .connecting(connState)
            case .reconnecting:
                state = .reconnecting(connState)
            }

            // Tunnel monitor should already be paused at this point so don't stop it to avoid a reset of its internal
            // counters.
            try? await reconnectInner(to: .random, shouldStopTunnelMonitor: false)

        case .initial, .disconnected, .disconnecting, .error:
            break
        }
    }
}
