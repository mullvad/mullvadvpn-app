//
//  RelayFilter.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-08.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct RelayFilter: Codable, Equatable {
    public enum Ownership: Codable {
        case any
        case owned
        case rented
    }

    public var ownership: Ownership
    public var providers: RelayConstraint<[String]>

    public init(ownership: Ownership = .any, providers: RelayConstraint<[String]> = .any) {
        self.ownership = ownership
        self.providers = providers
    }
}
