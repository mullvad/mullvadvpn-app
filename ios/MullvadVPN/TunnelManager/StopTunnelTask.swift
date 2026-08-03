//
//  StopTunnelTask.swift
//  MullvadVPN
//
//  Created by Mojgan on 15/12/2021.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

final class StopTunnelTask: @unchecked Sendable {
    private let interactor: TunnelInteractor
    private let isOnDemandEnabled: Bool

    init(interactor: TunnelInteractor, isOnDemandEnabled: Bool = false) {
        self.interactor = interactor
        self.isOnDemandEnabled = isOnDemandEnabled
    }

    func start() async throws {
        try Task.checkCancellation()
        switch interactor.tunnelStatus.state {
        case .disconnecting(.reconnect):
            interactor.updateTunnelStatus { tunnelStatus in
                tunnelStatus.state = .disconnecting(.nothing)
            }
        case .connected, .connecting, .reconnecting, .waitingForConnectivity(.noConnection), .error,
            .negotiatingEphemeralPeer:
            guard let tunnel = interactor.tunnel else {
                throw UnsetTunnelError()
            }
            try await shutdown(tunnel)
        default:
            break
        }

    }

    private func shutdown(_ tunnel: any TunnelProtocol) async throws {
        tunnel.isOnDemandEnabled = isOnDemandEnabled

        return try await withCheckedThrowingContinuation { continuation in
            tunnel.saveToPreferences { error in
                if let error {
                    continuation.resume(throwing: error)
                } else {
                    tunnel.stop()
                    continuation.resume(returning: ())
                }
            }
        }
    }
}
