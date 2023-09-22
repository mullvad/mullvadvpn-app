//
//  Actor+SleepCycle.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    /**
     Clients should call this method to notify actor when device wakes up.

     `NEPacketTunnelProvider` provides the corresponding lifecycle method.
     */
    public nonisolated func onWake() {
        tunnelMonitor.onWake()
    }

    /**
     Clients should call this method to notify actor when device is about to go to sleep.

     `NEPacketTunnelProvider` provides the corresponding lifecycle method.
     */
    public nonisolated func onSleep() {
        tunnelMonitor.onSleep()
    }
}
