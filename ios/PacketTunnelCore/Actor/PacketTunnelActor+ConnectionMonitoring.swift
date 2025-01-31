//
//  Actor+ConnectionMonitoring.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    func listenForTunnelMonitorEvents() async {
        tunnelMonitorTask?.cancel()

        tunnelMonitorTask = Task { [weak self] in
            guard let self else { return }
            for await event in self.tunnelMonitor.eventStream {
                self.eventChannel.send(.monitorEvent(event))
            }
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

        case .initial, .connected, .disconnecting, .disconnected, .error, .negotiatingEphemeralPeer:
            break
        }
    }

    /// Tell the tunnel to reconnect providing the correct reason to ensure that the attempt counter is incremented before reconnect.
    private func onHandleConnectionRecovery() async {
        switch state {
        case .connecting, .reconnecting, .connected:
            eventChannel.send(.reconnect(.random, reason: .connectionLoss))

        case .initial, .disconnected, .disconnecting, .error, .negotiatingEphemeralPeer:
            break
        }
    }
}
