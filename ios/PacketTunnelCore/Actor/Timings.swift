// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadTypes

/// Struct holding all timings used by tunnel actor.
public struct PacketTunnelActorTimings: Sendable {
    /// Periodicity at which actor will attempt to restart when an error occurred on system boot when filesystem is locked until device is unlocked or tunnel adapter error.
    public var bootRecoveryPeriodicity: Duration

    /// Time that takes for new WireGuard key to propagate across relays.
    public var wgKeyPropagationDelay: Duration

    /// Designated initializer.
    public init(
        bootRecoveryPeriodicity: Duration = .seconds(5),
        wgKeyPropagationDelay: Duration = .seconds(120)
    ) {
        self.bootRecoveryPeriodicity = bootRecoveryPeriodicity
        self.wgKeyPropagationDelay = wgKeyPropagationDelay
    }
}
