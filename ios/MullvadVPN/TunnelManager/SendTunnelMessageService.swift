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
import PacketTunnelCore

actor SendTunnelMessageService {
    private let tunnel: any TunnelProtocol

    init(tunnel: any TunnelProtocol, message: TunnelProviderMessage) {
        self.tunnel = tunnel
    }

    public func send<Output: Sendable & Codable>(message: TunnelProviderMessage) async throws -> Output {
        let result: Output = try await withCheckedThrowingContinuation { continuation in
            do {
                let messageData = try message.encode()
                try tunnel.sendProviderMessage(messageData) { reply in
                    do {
                        guard let reply else {
                            continuation.resume(throwing: SendTunnelProviderMessageError.tunnelDown(self.tunnel.status))
                            return
                        }
                        let output = try JSONDecoder().decode(Output.self, from: reply)
                        continuation.resume(returning: output)
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
}
