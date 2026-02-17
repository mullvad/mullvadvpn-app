//
//  TunnelViewControllerInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import MullvadTypes

final class TunnelViewControllerInteractor: @unchecked Sendable {
    private let tunnelManager: TunnelManager
    private let outgoingConnectionService: OutgoingConnectionServiceHandling
    private var tunnelObserver: TunnelObserver?
    private var outgoingConnectionTask: Task<Void, Error>?

    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)?
    var didUpdateDeviceState: ((_ deviceState: DeviceState, _ previousDeviceState: DeviceState) -> Void)?
    var didUpdateTunnelSettings: ((LatestTunnelSettings) -> Void)?
    var didGetOutgoingAddress: (@MainActor (OutgoingConnectionInfo) -> Void)?

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
        outgoingConnectionTask?.cancel()
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
                outgoingConnectionTask?.cancel()
                didUpdateTunnelStatus?(tunnelStatus)
                if case .connected = tunnelStatus.state {
                    outgoingConnectionTask = Task(priority: .high) { [weak self] in
                        guard
                            let outgoingConnectionInfo = try await self?.outgoingConnectionService
                                .getOutgoingConnectionInfo()
                        else {
                            return
                        }
                        await self?.didGetOutgoingAddress?(outgoingConnectionInfo)
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
