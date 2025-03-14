//
//  RelayFilterDataSourceItem.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct RelayFilterDataSourceItem: Hashable, Comparable {
    let name: String
    var description = ""
    let type: ItemType
    let isEnabled: Bool

    enum ItemType: Hashable {
        case ownershipAny, ownershipOwned, ownershipRented, allProviders, provider
    }

    static let anyOwnershipItem = RelayFilterDataSourceItem(name: NSLocalizedString(
        "RELAY_FILTER_ANY_LABEL",
        tableName: "RelayFilter",
        value: "Any",
        comment: ""
    ), type: .ownershipAny, isEnabled: true)

    static let ownedOwnershipItem = RelayFilterDataSourceItem(name: NSLocalizedString(
        "RELAY_FILTER_OWNED_LABEL",
        tableName: "RelayFilter",
        value: "Owned",
        comment: ""
    ), type: .ownershipOwned, isEnabled: true)

    static let rentedOwnershipItem = RelayFilterDataSourceItem(name: NSLocalizedString(
        "RELAY_FILTER_RENTED_LABEL",
        tableName: "RelayFilter",
        value: "Rented",
        comment: ""
    ), type: .ownershipRented, isEnabled: true)

    static let ownerships: [RelayFilterDataSourceItem] = [anyOwnershipItem, ownedOwnershipItem, rentedOwnershipItem]

    static var allProviders: RelayFilterDataSourceItem {
        RelayFilterDataSourceItem(name: NSLocalizedString(
            "RELAY_FILTER_ALL_PROVIDERS_LABEL",
            tableName: "RelayFilter",
            value: "All Providers",
            comment: ""
        ), type: .allProviders, isEnabled: true)
    }

    static func < (lhs: RelayFilterDataSourceItem, rhs: RelayFilterDataSourceItem) -> Bool {
        let nameComparison = lhs.name.caseInsensitiveCompare(rhs.name)
        return nameComparison == .orderedAscending
    }
}
