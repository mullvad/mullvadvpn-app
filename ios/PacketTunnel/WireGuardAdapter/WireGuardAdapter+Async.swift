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
import WireGuardKit

extension WireGuardAdapter {
    func start(tunnelConfiguration: TunnelConfiguration, daita: DaitaConfiguration?) async throws {
        return try await withCheckedThrowingContinuation { continuation in
            start(tunnelConfiguration: tunnelConfiguration, daita: daita) { error in
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume(returning: ())
                }
            }
        }
    }

    func startMultihop(
        entryConfiguration: TunnelConfiguration?,
        exitConfiguration: TunnelConfiguration,
        daita: DaitaConfiguration?
    ) async throws {
        return try await withCheckedThrowingContinuation { continuation in
            startMultihop(
                exitConfiguration: exitConfiguration,
                entryConfiguration: entryConfiguration,
                daita: daita
            ) { error in
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume(returning: ())
                }
            }
        }
    }

    func stop() async throws {
        return try await withCheckedThrowingContinuation { continuation in
            stop { error in
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume(returning: ())
                }
            }
        }
    }

    func update(tunnelConfiguration: TunnelConfiguration) async throws {
        return try await withCheckedThrowingContinuation { continuation in
            update(tunnelConfiguration: tunnelConfiguration) { error in
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume(returning: ())
                }
            }
        }
    }
}
