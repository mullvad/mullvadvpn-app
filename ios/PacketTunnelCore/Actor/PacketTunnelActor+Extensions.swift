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

extension PacketTunnelActor {
    /// Returns a stream yielding `ObservedState`.
    /// Note that the stream yields current value when created.
    public var observedStates: AsyncStream<ObservedState> {
        stateBroadcaster.makeStream(replaying: observedState)
    }

    /// Wait until the `observedState` moved to `.disconnected`.
    public func waitUntilDisconnected() async {
        await observedStates.waitUntilDisconnected()
    }
}
