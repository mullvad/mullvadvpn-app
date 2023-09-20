//
//  StoredAccountData.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2023-07-31.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct StoredAccountData: Codable, Equatable {
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
