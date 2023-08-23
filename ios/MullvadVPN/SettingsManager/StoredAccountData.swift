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
    }
}
