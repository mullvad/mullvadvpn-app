//
//  TunnelMonitorDelegate.swift
//  PacketTunnel
//
//  Created by pronebird on 15/07/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol TunnelMonitorDelegate: AnyObject {
    /// Invoked when tunnel monitor determined that connection is established.
    func tunnelMonitorDidDetermineConnectionEstablished(_ tunnelMonitor: TunnelMonitor)

    /// Invoked when tunnel monitor determined that connection attempt has failed.
    func tunnelMonitorDelegateShouldHandleConnectionRecovery(_ tunnelMonitor: TunnelMonitor)

    /// Invoked when network reachability status changes.
    func tunnelMonitor(
        _ tunnelMonitor: TunnelMonitor,
        networkReachabilityStatusDidChange isNetworkReachable: Bool
    )
}
