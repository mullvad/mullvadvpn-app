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
        let result: Output = try await withCheckedThrowingContinuation { continuation in
            do {
                let messageData = try message.encode()
                /// No cancellation checks intentionally.
                /// Otherwise there would be no easy way to send a cancellation message for a `ProxyAPIRequest`.
                try tunnel.sendProviderMessage(messageData) { reply in
                    do {
                        guard let reply else {
                            continuation.resume(throwing: SendTunnelProviderMessageError.tunnelDown(self.tunnel.status))
                            return
                        }
                        let output = try JSONDecoder().decode(TunnelProviderReply<Output>.self, from: reply)
                        continuation.resume(returning: output.value)
                    } catch {
                        continuation.resume(throwing: error)
                    }
                }
            } catch {
                continuation.resume(throwing: error)
            }
        }

        return result
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
    case tunnelDown(NEVPNStatus)
    var errorDescription: String? {
        switch self {
        case let .tunnelDown(status):
            return "Tunnel is either down or about to go down (status: \(status))."
        }
    }
}
