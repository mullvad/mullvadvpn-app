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

public struct StoredWgKeyData: Codable, Equatable, Sendable {
    /// Private key creation date.
    public var creationDate: Date

    /// Last date a rotation was attempted. Nil if last attempt was successful.
    public var lastRotationAttemptDate: Date?

    /// Private key.
    public var privateKey: WireGuard.PrivateKey

    /// Next private key we're trying to rotate to.
    /// Added in 2023.3
    public var nextPrivateKey: WireGuard.PrivateKey?

    public init(
        creationDate: Date,
        lastRotationAttemptDate: Date? = nil,
        privateKey: WireGuard.PrivateKey,
        nextPrivateKey: WireGuard.PrivateKey? = nil
    ) {
        self.creationDate = creationDate
        self.lastRotationAttemptDate = lastRotationAttemptDate
        self.privateKey = privateKey
        self.nextPrivateKey = nextPrivateKey
    }
}
