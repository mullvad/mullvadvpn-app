//
//  AppMessageHandler.swift
//  PacketTunnel
//
//  Created by pronebird on 19/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import PacketTunnelCore

/**
 Actor handling packet tunnel IPC (app) messages and patching them through to the right facility.
 */
struct AppMessageHandler {
    private let logger = Logger(label: "AppMessageHandler")
    private let packetTunnelActor: PacketTunnelActor
    private let urlRequestProxy: URLRequestProxy

    init(packetTunnelActor: PacketTunnelActor, urlRequestProxy: URLRequestProxy) {
        self.packetTunnelActor = packetTunnelActor
        self.urlRequestProxy = urlRequestProxy
    }

    /**
     Handle app message received via packet tunnel IPC.

     - Message data is expected to be a serialized `TunnelProviderMessage`.
     - Reply is expected to be wrapped in `TunnelProviderReply`.
     - Return `nil` in the event of error or when the call site does not expect any reply.

     Calls to reconnect and notify actor when private key is changed are meant to run in parallel because those tasks are serialized in `TunnelManager` and await
     the acknowledgment from IPC before starting next operation, hence it's critical to return as soon as possible.
     (See `TunnelManager.reconnectTunnel()`, `SendTunnelProviderMessageOperation`)
     */
    func handleAppMessage(_ messageData: Data) async -> Data? {
        guard let message = decodeMessage(messageData) else { return nil }

        logger.debug("Received app message: \(message)")

        switch message {
        case let .sendURLRequest(request):
            return await encodeReply(urlRequestProxy.sendRequest(request))

        case let .cancelURLRequest(id):
            urlRequestProxy.cancelRequest(identifier: id)
            return nil

        case .getTunnelStatus:
            return await encodeReply(packetTunnelActor.state.packetTunnelStatus)

        case .privateKeyRotation:
            packetTunnelActor.notifyKeyRotation(date: nil)
            return nil

        case let .reconnectTunnel(nextRelay):
            packetTunnelActor.reconnect(to: nextRelay)
            return nil
        }
    }

    /// Deserialize `TunnelProviderMessage` or return `nil` on error. Errors are logged but ignored.
    private func decodeMessage(_ data: Data) -> TunnelProviderMessage? {
        do {
            return try TunnelProviderMessage(messageData: data)
        } catch {
            logger.error(error: error, message: "Failed to decode the app message.")
            return nil
        }
    }

    /// Encode `TunnelProviderReply` or return `nil` on error. Errors are logged but ignored.
    private func encodeReply<T: Codable>(_ reply: T) -> Data? {
        do {
            return try TunnelProviderReply(reply).encode()
        } catch {
            logger.error(error: error, message: "Failed to encode the app message reply.")
            return nil
        }
    }
}
