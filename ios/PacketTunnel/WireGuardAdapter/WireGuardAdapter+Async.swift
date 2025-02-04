//
//  WireGuardAdapter+Async.swift
//  PacketTunnel
//
//  Created by pronebird on 30/06/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
