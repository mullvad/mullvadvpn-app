//
//  TransportMonitor.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-07.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

class TransportMonitor {
    static let shared = TransportMonitor()

    let registry = RESTTransportRegistry()

    private let packetTunnelTransport = PacketTunnelTransport()

    private init() {
        TunnelManager.shared.addObserver(self)

        register(
            URLSessionTransport(urlSession: REST.sharedURLSession)
        )
    }

    func register(_ transport: RESTTransport) {
        registry.register(transport)
    }

    func unregister(_ transport: RESTTransport) {
        registry.unregister(transport)
    }

    func getTransport() -> RESTTransport? {
        registry.getTransport()
    }
}

extension TransportMonitor: TunnelObserver {
    func transportDidTimeout(_ transport: RESTTransport, maxRetryStrategy: Int) {
        registry.transportDidTimeout(transport, maxRetryStrategy: maxRetryStrategy)
    }

    func transportDidFinishLoad(_ transport: RESTTransport) {
        registry.transportDidFinishLoad(transport)
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelStatus tunnelStatus: TunnelStatus) {
        if shouldUsePacketTunnelTransport(state: tunnelStatus.state) {
            registry.register(packetTunnelTransport)
        }
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateDeviceState deviceState: DeviceState) {
        if deviceState == .revoked {
            registry.register(packetTunnelTransport)
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
