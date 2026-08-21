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
import MullvadLogging
import MullvadREST
import MullvadTypes

final class APITransportMonitor: APITransportProviderProtocol {
    private let tunnelManager: TunnelManager
    private let tunnelStore: TunnelStore
    private let requestFactory: MullvadApiRequestFactory

    init(tunnelManager: TunnelManager, tunnelStore: TunnelStore, requestFactory: MullvadApiRequestFactory) {
        self.tunnelManager = tunnelManager
        self.tunnelStore = tunnelStore
        self.requestFactory = requestFactory
    }

    func makeTransport() -> APITransportProtocol? {
        let tunnel = tunnelStore.getPersistentTunnels().first { tunnel in
            tunnel.status == .connecting || tunnel.status == .reasserting || tunnel.status == .connected
        }

        return if let tunnel, shouldRouteThroughTunnel(tunnel: tunnel) {
            PacketTunnelAPITransport(tunnel: tunnel)
        } else {
            APITransport(requestFactory: requestFactory)
        }
    }

    /// Determines whether the tunnel tunnel should be used to pipe requests,
    ///
    /// - Parameter tunnel: The tunnel tunnel to evaluate
    /// - Returns: `true` if the tunnel should be used; otherwise, `false`
    private func shouldRouteThroughTunnel(tunnel: any TunnelProtocol) -> Bool {
        switch tunnel.status {
        case .connected:
            switch tunnelManager.tunnelStatus.state {
            // Use tunnel if the tunnel is connected but the tunnel manager
            // reports an error or the tunnel is still connecting
            case .error, .connecting, .negotiatingEphemeralPeer:
                true
            // Also use tunnel if configuration is loaded and device is revoked
            default:
                tunnelManager.isConfigurationLoaded && tunnelManager.deviceState == .revoked
            }

        case .connecting, .reasserting:
            // Use tunnel while it's in a transitional connecting state
            true

        default:
            // In all other cases, do not use the tunnel
            false
        }
    }
}
