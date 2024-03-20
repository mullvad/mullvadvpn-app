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
            // TODO: switch here on whether we need to do PQ negotiation
            let isPostQuantum = true
            state = isPostQuantum ? .negotiatingKey(connState) : .connected(connState)

        case .initial, .connected, .disconnecting, .disconnected, .error, .negotiatingPostQuantumKey:
            break
        }
    }

    /// Tell the tunnel to reconnect providing the correct reason to ensure that the attempt counter is incremented before reconnect.
    private func onHandleConnectionRecovery() async {
        switch state {
        case .connecting, .reconnecting, .connected, .negotiatingKey:
            commandChannel.send(.reconnect(.random, reason: .connectionLoss))

        case .initial, .disconnected, .disconnecting, .error, .negotiatingPostQuantumKey:
            break
        }
    }
}
