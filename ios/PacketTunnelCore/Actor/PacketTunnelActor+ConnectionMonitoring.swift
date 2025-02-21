//
//  Actor+ConnectionMonitoring.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    /// Assign a closure receiving tunnel monitor events.
    func setTunnelMonitorEventHandler() {
        tunnelMonitor.onEvent = { [weak self] event in
            /// Dispatch tunnel monitor events via command channel to guarantee the order of execution.
            self?.eventChannel.send(.monitorEvent(event))
        }
    }
}
