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
