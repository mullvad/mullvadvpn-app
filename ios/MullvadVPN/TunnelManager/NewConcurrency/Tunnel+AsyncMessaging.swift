//
//  Tunnel+AsyncMessaging.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-19.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import Foundation
import MullvadREST
import MullvadTypes
import PacketTunnelCore

extension TunnelProtocol {
    /// Request packet tunnel process to reconnect the tunnel with the given relays.
    func reconnectTunnel(to nextRelays: NextRelays) async throws -> ObservedState {
        let task = SendTunnelProviderMessageTask(tunnel: self, message: .reconnectTunnel(nextRelays))
        let result = try await task.start()
        return try mapObservedState(data: result)
    }
    
    /// Request status from packet tunnel process.
    func getTunnelStatus() async throws -> ObservedState {
        let task = SendTunnelProviderMessageTask(tunnel: self, message: .getTunnelStatus)
        let result = try await task.start()
        return try mapObservedState(data: result)
    }
    
    
    func mapObservedState(data: Data?) throws -> ObservedState {
        if let data {
            return try TunnelProviderReply<ObservedState>(messageData: data).value
        } else {
            throw EmptyTunnelProviderResponseError()
        }
    }
}
