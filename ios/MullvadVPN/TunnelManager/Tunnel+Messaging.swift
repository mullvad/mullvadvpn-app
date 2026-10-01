// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadREST
import MullvadTypes
import Operations
import PacketTunnelCore

extension TunnelProtocol {
    /// Request packet tunnel process to reconnect the tunnel with the given relays.
    func reconnectTunnel(
        to nextRelays: NextRelays,
        completionHandler: @escaping @Sendable (Result<ObservedState, Error>) -> Void
    ) -> Cancellable {
        let messageService = SendTunnelMessageService(tunnel: self)
        return messageService.send(message: .reconnectTunnel(nextRelays), completionHandler: completionHandler)
    }
    /// Request status from packet tunnel process.
    func getTunnelStatus(
        completionHandler: @escaping @Sendable (Result<ObservedState, Error>) -> Void
    ) -> Cancellable {
        let messageService = SendTunnelMessageService(tunnel: self)
        return messageService.send(message: .getTunnelStatus, completionHandler: completionHandler)
    }

    /// Send API request via packet tunnel process bypassing VPN.
    func sendAPIRequest(
        _ proxyRequest: ProxyAPIRequest,
        completionHandler: @escaping @Sendable (Result<ProxyAPIResponse, Error>) -> Void
    ) -> Cancellable {
        let messageService = SendTunnelMessageService(tunnel: self)

        let cancelHandler = { @Sendable in
            _ = messageService.send(message: .cancelAPIRequest(proxyRequest.id)) {
                (reply: Result<TunnelReply, Error>) -> Void in
                // Do nothing with the answer
            }
        }

        return messageService.send(
            message: .sendAPIRequest(proxyRequest),
            completionHandler: completionHandler,
            cancelHandler: cancelHandler)
    }

    /// Notify tunnel about private key rotation.
    func notifyKeyRotation(
        completionHandler: @escaping @Sendable (Result<TunnelReply, Error>) -> Void
    ) -> Cancellable {
        let messageService = SendTunnelMessageService(tunnel: self)
        return messageService.send(message: .privateKeyRotation, completionHandler: completionHandler)
    }

}
