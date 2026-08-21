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

@testable import MullvadRustRuntime
@testable import MullvadTypes

struct TunnelObfuscationStub: TunnelObfuscation {
    var transportLayer: TransportLayer { .udp }

    let remotePort: UInt16
    init(
        remoteAddress: IPAddress,
        remotePort: UInt16,
        obfuscationProtocol: TunnelObfuscationProtocol
    ) {
        self.remotePort = remotePort
    }

    func start() {}

    func stop() {}

    var localUdpPort: UInt16 { 42 }
}
