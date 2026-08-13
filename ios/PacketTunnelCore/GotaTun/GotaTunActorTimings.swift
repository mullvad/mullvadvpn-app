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

/// Timing configuration for the GotaTun actor.
public struct GotaTunActorTimings: Sendable {
    /// How long to wait after a key rotation before switching to the new key.
    public let wgKeyPropagationDelay: Duration

    public init(
        wgKeyPropagationDelay: Duration = .seconds(120)
    ) {
        self.wgKeyPropagationDelay = wgKeyPropagationDelay
    }
}
