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

/// Determines which WireGuard private key a connection should use across key
/// rotation.
///
/// When a new key is uploaded to the API, it can take minutes before it
/// propagates to all relays. Until then, the old key should be used.
struct GotaTunKeyPolicy: Sendable {
    /// The key the current adapter was configured with.
    private var activeKey: WireGuard.PrivateKey?
    /// The pre-rotation key, kept in use until the new one has propagated to the relay.
    private var priorKey: WireGuard.PrivateKey?

    /// Returns the key a new connection should use, recording it as active: the pre-rotation
    /// key while it is still propagating, otherwise `settingsKey`.
    mutating func connectionKey(settingsKey: WireGuard.PrivateKey) -> WireGuard.PrivateKey {
        let key = priorKey ?? settingsKey
        activeKey = key
        return key
    }

    /// Marks the device key as rotated, keeping the currently connected key in use
    /// until propagation completes.
    mutating func beginRotation() {
        priorKey = activeKey
    }

    /// Ends the propagation window; subsequent connections use the settings key.
    mutating func endRotation() {
        priorKey = nil
    }
}
