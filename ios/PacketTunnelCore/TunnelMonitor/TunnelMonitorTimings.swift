//
//  TunnelMonitorTimings.swift
//  PacketTunnelCore
//
//  Created by Jon Petersson on 2023-09-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public struct TunnelMonitorTimings: Sendable {
    /// Interval for periodic heartbeat ping issued when traffic is flowing.
    /// Should help to detect connectivity issues on networks that drop traffic in one of directions,
    /// regardless if tx/rx counters are being updated.
    let heartbeatPingInterval: Duration

    /// Heartbeat timeout that once exceeded triggers next heartbeat to be sent.
    let heartbeatReplyTimeout: Duration

    /// Timeout used to determine if there was a network activity lately.
    var trafficFlowTimeout: Duration { heartbeatPingInterval * 0.5 }

    /// Ping timeout.
    let pingTimeout: Duration

    /// Interval to wait before sending next ping.
    let pingDelay: Duration

    /// Initial timeout when establishing connection.
    let initialEstablishTimeout: Duration

    /// Multiplier applied to `establishTimeout` on each failed connection attempt.
    let establishTimeoutMultiplier: UInt32

    /// Maximum timeout when establishing connection.
    var maxEstablishTimeout: Duration { pingTimeout }

    /// Connectivity check periodicity.
    let connectivityCheckInterval: Duration

    /// Inbound traffic timeout used when outbound traffic was registered prior to inbound traffic.
    let inboundTrafficTimeout: Duration

    /// Traffic timeout applied when both tx/rx counters remain stale, i.e no traffic flowing.
    /// Ping is issued after that timeout is exceeded.s
    let trafficTimeout: Duration

    public init(
        heartbeatPingInterval: Duration = .seconds(10),
        heartbeatReplyTimeout: Duration = .seconds(3),
        pingTimeout: Duration = .seconds(15),
        pingDelay: Duration = .seconds(3),
        initialEstablishTimeout: Duration = .seconds(4),
        establishTimeoutMultiplier: UInt32 = 2,
        connectivityCheckInterval: Duration = .seconds(1),
        inboundTrafficTimeout: Duration = .seconds(5),
        trafficTimeout: Duration = .minutes(2)
    ) {
        self.heartbeatPingInterval = heartbeatPingInterval
        self.heartbeatReplyTimeout = heartbeatReplyTimeout
        self.pingTimeout = pingTimeout
        self.pingDelay = pingDelay
        self.initialEstablishTimeout = initialEstablishTimeout
        self.establishTimeoutMultiplier = establishTimeoutMultiplier
        self.connectivityCheckInterval = connectivityCheckInterval
        self.inboundTrafficTimeout = inboundTrafficTimeout
        self.trafficTimeout = trafficTimeout
    }
}
