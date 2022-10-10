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
    private let urlSessionTunnelTransport = URLSessionTransport(urlSession: REST.sharedURLSession)

    init() {
        TunnelManager.shared.addObserver(self)

        RESTTransportRegistry.shared.register(
            urlSessionTunnelTransport
        )

        RESTTransportRegistry.shared.register(
            packetTunnelTransport
        )
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelStatus tunnelStatus: TunnelStatus) {
        RESTTransportRegistry.shared.setTransports(
            stateUpdated(tunnelState: tunnelStatus.state, deviceState: manager.deviceState)
        )
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateDeviceState deviceState: DeviceState) {
        RESTTransportRegistry.shared.setTransports(
            stateUpdated(tunnelState: manager.tunnelStatus.state, deviceState: deviceState)
        )
    }

    private func stateUpdated(
        tunnelState: TunnelState,
        deviceState: DeviceState
    ) -> [RESTTransport] {
        switch (tunnelState, deviceState) {
        case (.connected, .revoked):
            return [packetTunnelTransport]

        case (.pendingReconnect, _):
            return [urlSessionTunnelTransport]

        case (.waitingForConnectivity, _):
            return [urlSessionTunnelTransport]

        case (.connecting, _):
            return [packetTunnelTransport]

        case (.reconnecting, _):
            return [packetTunnelTransport]

        case (.disconnecting, _):
            return [urlSessionTunnelTransport]

        case (.disconnected, _):
            return [urlSessionTunnelTransport]

        case (.connected, _):
            return [urlSessionTunnelTransport]
        }
    }

    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager) {}

    func tunnelManager(
        _ manager: TunnelManager,
        didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2
    ) {}

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error) {}
}
