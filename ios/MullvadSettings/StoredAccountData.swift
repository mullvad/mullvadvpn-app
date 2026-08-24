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

public struct StoredAccountData: Codable, Equatable, Sendable {
    /// Account identifier.
    public var identifier: String

    /// Account number.
    public var number: String

    /// Account expiry.
    public var expiry: Date

    /// Returns `true` if account has expired.
    public var isExpired: Bool {
        expiry <= Date()
    }

    public init(identifier: String, number: String, expiry: Date) {
        self.identifier = identifier
        self.number = number
        self.expiry = expiry
    }
}

extension StoredAccountData {
    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.identifier = try container.decode(String.self, forKey: .identifier)
        self.number = try container.decode(String.self, forKey: .number)
        self.expiry = try container.decode(Date.self, forKey: .expiry)
    }
}
