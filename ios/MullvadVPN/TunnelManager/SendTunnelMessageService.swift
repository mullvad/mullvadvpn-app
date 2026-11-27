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
import MullvadTypes
import NetworkExtension
import PacketTunnelCore

actor SendTunnelMessageService {
    typealias SendableData = Sendable & Codable
    private let tunnel: any TunnelProtocol

    init(tunnel: any TunnelProtocol) {
        self.tunnel = tunnel
    }

    public func send<Output: SendableData>(message: TunnelProviderMessage) async throws -> Output {
        let messageData = try message.encode()
        let reply = try await tunnel.sendProviderMessage(messageData)
        guard let reply else {
            throw SendTunnelProviderMessageError.unexpectedNilReply
        }
        let output = try JSONDecoder().decode(TunnelProviderReply<Output>.self, from: reply)
        return output.value
    }

    nonisolated public func send<Output: SendableData>(
        message: TunnelProviderMessage,
        completionHandler: @escaping @Sendable (Result<Output, Error>) -> Void,
        cancelHandler: (@Sendable () -> Void)? = nil
    ) -> Cancellable {
        let task = Task {
            do {
                let reply: Output = try await send(message: message)
                completionHandler(.success(reply))
            } catch {
                completionHandler(.failure(error))
            }
        }

        return AnyCancellable {
            task.cancel()
            cancelHandler?()
        }
    }
}

enum SendTunnelProviderMessageError: LocalizedError {
    /// Tunnel process is either down or about to go down.
    case unexpectedNilReply
    case tunnelDown(NEVPNStatus)
    var errorDescription: String? {
        switch self {
        case .unexpectedNilReply:
            return "Tunnel couldn't encode reply, check PacketTunnel logs"
        case let .tunnelDown(status):
            return "Tunnel is either down or about to go down (status: \(status))."
        }
    }
}
