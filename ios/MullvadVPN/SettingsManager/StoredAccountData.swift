//
//  StoredAccountData.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-07-31.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct StoredAccountData: Codable, Equatable {
    /// Account identifier.
    var identifier: String

    /// Account number.
    var number: String

    /// Account expiry.
    var expiry: Date

    /// Set to `true` when the account is created and flipped to `false` when the user adds more credit.
    var isNew = false

    /// Returns `true` if account has expired.
    var isExpired: Bool {
        expiry <= Date()
    }
}

extension StoredAccountData {
    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        self.identifier = try container.decode(String.self, forKey: .identifier)
        self.number = try container.decode(String.self, forKey: .number)
        self.expiry = try container.decode(Date.self, forKey: .expiry)

        // When the app is upgraded from 2023.3 or below, this field won't exist, and the auto synthesized init will fail.
        // This leads to a reset of the settings. If the key isn't present, consider the account not new to avoid the issue.
        let isNewAccount = try? container.decode(Bool.self, forKey: .isNew)
        self.isNew = isNewAccount ?? false
    }
}
