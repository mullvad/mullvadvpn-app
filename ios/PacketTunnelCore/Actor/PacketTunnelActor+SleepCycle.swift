//
//  Actor+SleepCycle.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    /**
     Clients should call this method to notify actor when device wakes up.

     `NEPacketTunnelProvider` provides the corresponding lifecycle method.
     */
    public func onWake() async {
        await tunnelMonitor.wake()
    }

    /**
     Clients should call this method to notify actor when device is about to go to sleep.

     `NEPacketTunnelProvider` provides the corresponding lifecycle method.
     */
    public func onSleep() async {
        await tunnelMonitor.sleep()
    }
}
