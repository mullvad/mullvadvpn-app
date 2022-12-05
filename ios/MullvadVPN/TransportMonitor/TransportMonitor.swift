//
//  TransportMonitor.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-10-07.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import NetworkExtension

class TransportMonitor: TunnelObserver {
    private let tunnelManager: TunnelManager
    private var packetTunnelTransport: PacketTunnelTransport!
    private let urlSessionTransport: REST.URLSessionTransport

    private let nslock = NSLock()
    private var _transport: RESTTransport?
    private var vpnStatus: NEVPNStatus?

    var transport: RESTTransport? {
        nslock.lock()
        defer { nslock.unlock() }

        return _transport
    }

    init(tunnelManager: TunnelManager, tunnel: TunnelProviderManagerType?) {
        self.tunnelManager = tunnelManager

        urlSessionTransport = REST.URLSessionTransport(urlSession: REST.makeURLSession())

        if let tunnel = tunnel {
            NotificationCenter.default.addObserver(
                self, selector: #selector(handleVPNStatusChangeNotification),
                name: .NEVPNStatusDidChange,
                object: tunnel.connection
            )

            packetTunnelTransport = PacketTunnelTransport(
                tunnel: Tunnel(tunnelProvider: tunnel)
            )

            vpnStatus = tunnel.connection.status
        }

        NotificationCenter.default.addObserver(
            self, selector: #selector(handleVPNConfigurationChangeNotification),
            name: .NEVPNConfigurationChange,
            object: nil
        )

        tunnelManager.addObserver(self)
        setTransports()
    }

    // MARK: - VPNObserver

    @objc private func handleVPNStatusChangeNotification(_ notification: Notification) {
        guard let connection = notification.object as? VPNConnectionProtocol else { return }

        vpnStatus = connection.status
        setTransports()
    }

    @objc private func handleVPNConfigurationChangeNotification(_ notification: Notification) {
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
        nslock.lock()
        defer { nslock.unlock() }

        _transport = stateUpdated(
            tunnelState: tunnelManager.tunnelStatus.state,
            deviceState: tunnelManager.deviceState
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
            if let packetTunnelTransport = packetTunnelTransport, vpnStatus == .connecting {
                return packetTunnelTransport
            }

            return urlSessionTransport

        case (.connected, _):
            return urlSessionTransport
        }
    }
}
