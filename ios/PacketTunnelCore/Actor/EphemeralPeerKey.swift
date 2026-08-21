// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes

/// The preshared / private key  used by ephemeral peers
public struct EphemeralPeerKey: Equatable {
    public let preSharedKey: WireGuard.PreSharedKey?
    public let ephemeralKey: WireGuard.PrivateKey

    public init(preSharedKey: WireGuard.PreSharedKey? = nil, ephemeralKey: WireGuard.PrivateKey) {
        self.preSharedKey = preSharedKey
        self.ephemeralKey = ephemeralKey
    }
}
