//
//  TunnelMonitorProtocol.swift
//  PacketTunnelCore
//
//  Created by pronebird on 10/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

/// Tunnel monitor event.
public enum TunnelMonitorEvent {
    /// Dispatched after receiving the first ping response
    case connectionEstablished

    /// Dispatched when connection stops receiving ping responses.
    /// The handler is responsible to reconfigure the tunnel and call `TunnelMonitorProtocol.start(probeAddress:)` to resume connection monitoring.
    case connectionLost
}

/// A type that can provide tunnel monitoring.
public protocol TunnelMonitorProtocol: AnyObject {
    /// Event handler that starts receiving events after the call to `start(probeAddress:)`.
    var onEvent: ((TunnelMonitorEvent) -> Void)? { get set }

    /// Start monitoring connection by pinging the given IP address.
    /// Normally we should only give an address of a tunnel gateway here which is reachable over tunnel interface.
    func start(probeAddress: IPv4Address)

    /// Stop monitoring connection.
    func stop()

    /// Restarts internal timers and gracefully handles transition from sleep to awake device state.
    /// Call this method when packet tunnel provider receives a wake event.
    func onWake()

    /// Cancels internal timers and time dependent data in preparation for device sleep.
    /// Call this method when packet tunnel provider receives a sleep event.
    func onSleep()

    /// Handle changes in network path, eg. update connection state and monitoring.
    func handleNetworkPathUpdate(_ networkPath: NetworkPath)
}
