//
//  Timings.swift
//  PacketTunnelCore
//
//  Created by pronebird on 21/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Struct holding all timings used by tunnel actor.
public struct PacketTunnelActorTimings {
    /// Periodicity at which actor will attempt to restart when an error occurred on system boot when filesystem is locked until device is unlocked.
    public var bootRecoveryPeriodicity: Duration

    /// Time that takes for new WireGuard key to propagate across relays.
    public var wgKeyPropagationDelay: Duration

    /// Designated initializer.
    public init(
        bootRecoveryPeriodicity: Duration = .seconds(10),
        wgKeyPropagationDelay: Duration = .seconds(120)
    ) {
        self.bootRecoveryPeriodicity = bootRecoveryPeriodicity
        self.wgKeyPropagationDelay = wgKeyPropagationDelay
    }
}
