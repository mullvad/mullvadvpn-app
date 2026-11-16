// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import MullvadSettings
import MullvadTypes

final class TunnelViewControllerInteractor: @unchecked Sendable {
    private let tunnelManager: TunnelManager
    private let outgoingConnectionService: OutgoingConnectionServiceHandling
    private var tunnelObserver: TunnelObserver?
    private var outgoingIPv4ConnectionTask: Task<Void, Error>?
    private var outgoingIPv6ConnectionTask: Task<Void, Error>?

    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)?
    var didUpdateDeviceState: ((_ deviceState: DeviceState, _ previousDeviceState: DeviceState) -> Void)?
    var didUpdateTunnelSettings: ((LatestTunnelSettings) -> Void)?
    var didGetOutgoingIPv4Address: (@MainActor (IPV4ConnectionData) -> Void)?
    var didGetOutgoingIPv6Address: (@MainActor (IPV6ConnectionData) -> Void)?

    var tunnelStatus: TunnelStatus {
        tunnelManager.tunnelStatus
    }

    var deviceState: DeviceState {
        tunnelManager.deviceState
    }

    var tunnelSettings: LatestTunnelSettings {
        tunnelManager.settings
    }

    deinit {
        outgoingIPv4ConnectionTask?.cancel()
        outgoingIPv6ConnectionTask?.cancel()
    }

    init(
        tunnelManager: TunnelManager,
        outgoingConnectionService: OutgoingConnectionServiceHandling
    ) {
        self.tunnelManager = tunnelManager
        self.outgoingConnectionService = outgoingConnectionService

        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                guard let self else { return }
                outgoingIPv4ConnectionTask?.cancel()
                didUpdateTunnelStatus?(tunnelStatus)
                if case .connected = tunnelStatus.state {
                    outgoingIPv4ConnectionTask = Task { [weak self] in
                        guard
                            let outgoingConnectionInfo = try await self?.outgoingConnectionService
                                .getOutgoingIPv4ConnectionInfo()
                        else {
                            return
                        }
                        await self?.didGetOutgoingIPv4Address?(outgoingConnectionInfo)
                    }
                    outgoingIPv6ConnectionTask = Task { [weak self] in
                        guard
                            let outgoingConnectionInfo = try await self?.outgoingConnectionService
                                .getOutgoingIPv6ConnectionInfo()
                        else {
                            return
                        }
                        await self?.didGetOutgoingIPv6Address?(outgoingConnectionInfo)
                    }
                }
            },
            didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                self?.didUpdateDeviceState?(deviceState, previousDeviceState)
            },
            didUpdateTunnelSettings: { [weak self] _, tunnelSettings in
                self?.didUpdateTunnelSettings?(tunnelSettings)
            }
        )

        tunnelManager.addObserver(tunnelObserver)

        self.tunnelObserver = tunnelObserver
    }

    func startTunnel() {
        tunnelManager.startTunnel()
    }

    func stopTunnel() {
        tunnelManager.stopTunnel()
    }

    func reconnectTunnel(selectNewRelay: Bool) {
        tunnelManager.reconnectTunnel(selectNewRelay: selectNewRelay)
    }
}
