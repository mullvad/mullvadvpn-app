//
//  TransportMonitor.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-07.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

class TransportMonitor: TunnelObserver {
    private let packetTunnelTransport = PacketTunnelTransport()
    private let urlSessionTransport = URLSessionTransport(urlSession: REST.sharedURLSession)

    init() {
        TunnelManager.shared.addObserver(self)

        setTransports()
    }

    // MARK: - TunnelObserver

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelStatus tunnelStatus: TunnelStatus) {
        setTransports()
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateDeviceState deviceState: DeviceState) {
        setTransports()
    }

    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager) {
        setTransports()
    }

    func tunnelManager(
        _ manager: TunnelManager,
        didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2
    ) {}

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error) {}

    // MARK: - Private

    private func setTransports() {
        RESTTransportRegistry.shared.setTransport(
            stateUpdated(
                tunnelState: TunnelManager.shared.tunnelStatus.state,
                deviceState: TunnelManager.shared.deviceState
            )
        )
    }

    private func stateUpdated(
        tunnelState: TunnelState,
        deviceState: DeviceState
    ) -> RESTTransport {
        switch (tunnelState, deviceState) {
        case (.connected, .revoked):
            return packetTunnelTransport

        case (.pendingReconnect, _):
            return urlSessionTransport

        case (.waitingForConnectivity, _):
            return urlSessionTransport

        case (.connecting, _):
            return packetTunnelTransport

        case (.reconnecting, _):
            return packetTunnelTransport

        case (.disconnecting, _):
            return urlSessionTransport

        case (.disconnected, _):
            return urlSessionTransport

        case (.connected, _):
            return urlSessionTransport
        }
    }
}
