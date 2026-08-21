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
import Network

struct LocalNetworkProbe {
    /// Does a best effort attempt to trigger the local network privacy alert.
    func triggerLocalNetworkPrivacyAlert() {
        let dispatchQueue = DispatchQueue(label: "com.mullvad.localNetworkAlert")
        let localIpv4Connection = NWConnection(
            to: NWEndpoint.hostPort(host: .ipv4(.broadcast), port: .any),
            using: .udp
        )
        localIpv4Connection.start(queue: dispatchQueue)

        let localIpv6Connection = NWConnection(
            to: NWEndpoint.hostPort(host: .ipv6(.broadcast), port: .any),
            using: .udp
        )
        localIpv6Connection.start(queue: dispatchQueue)
    }
}
