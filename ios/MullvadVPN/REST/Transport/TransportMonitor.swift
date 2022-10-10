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

    private init() {
        TunnelManager.shared.addObserver(self)

        RESTTransportRegistry.shared.register(
            URLSessionTransport(urlSession: REST.sharedURLSession)
        )
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelStatus tunnelStatus: TunnelStatus) {
        if shouldUsePacketTunnelTransport(state: tunnelStatus.state) {
            RESTTransportRegistry.shared.register(packetTunnelTransport)
        }
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateDeviceState deviceState: DeviceState) {
        if deviceState == .revoked {
            RESTTransportRegistry.shared.register(packetTunnelTransport)
        }
    }

    private func shouldUsePacketTunnelTransport(state: TunnelState) -> Bool {
        switch state {
        case .connecting, .reconnecting: return true
        default: return false
        }
    }

    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager) {}

    func tunnelManager(
        _ manager: TunnelManager,
        didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2
    ) {}

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error) {}
}
