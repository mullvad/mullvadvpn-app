// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
@testable import PacketTunnelCore

struct ProtocolObfuscationStub: ProtocolObfuscation {
    var remotePort: UInt16 { 42 }

    func obfuscate(
        _ endpoint: MullvadTypes.SelectedEndpoint,
        clientPublicKey: WireGuard.PublicKey
    ) -> ProtocolObfuscationResult {
        .init(endpoint: endpoint)
    }

    var transportLayer: TransportLayer? { .udp }
}
