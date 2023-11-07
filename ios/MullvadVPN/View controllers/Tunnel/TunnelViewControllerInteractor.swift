//
//  TunnelViewControllerInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

final class TunnelViewControllerInteractor {
    private let tunnelManager: TunnelManager
    private let outgoingConnectionService: OutgoingConnectionServiceHandling
    private var tunnelObserver: TunnelObserver?
    private var outgoingConnectionTask: Task<Void, Error>?

    var didUpdateTunnelStatus: ((TunnelStatus) -> Void)?
    var didUpdateDeviceState: ((_ deviceState: DeviceState, _ previousDeviceState: DeviceState) -> Void)?
    var didGetOutGoingAddress: (@MainActor (OutgoingConnectionInfo) -> Void)?

    var tunnelStatus: TunnelStatus {
        tunnelManager.tunnelStatus
    }

    var deviceState: DeviceState {
        tunnelManager.deviceState
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
                switch tunnelStatus.state {
                case .connected:
                    outgoingConnectionTask = Task { [weak self] in
                        guard let outgoingConnectionInfo = try await self?.outgoingConnectionService
                            .getOutgoingConnectionInfo() else {
                            return
                        }
                        await self?.didGetOutGoingAddress?(outgoingConnectionInfo)
                    }
                    fallthrough
                default:
                    didUpdateTunnelStatus?(tunnelStatus)
                }
            },
            didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                self?.didUpdateDeviceState?(deviceState, previousDeviceState)
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
