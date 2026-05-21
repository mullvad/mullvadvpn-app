//
//  RelayFilterDataSourceItem.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension RelayFilterSelection.DataSource {
    struct Item: Hashable, Comparable {
        let name: String
        var description = ""
        let type: ItemType
        let isEnabled: Bool

        enum ItemType: Hashable {
            case ownershipAny, ownershipOwned, ownershipRented, allProviders, provider
        }

        static let anyOwnershipItem = Item(
            name: NSLocalizedString("Any", comment: ""),
            type: .ownershipAny,
            isEnabled: true
        )

        static let ownedOwnershipItem = Item(
            name: NSLocalizedString("Mullvad owned only", comment: ""),
            type: .ownershipOwned,
            isEnabled: true
        )

        static let rentedOwnershipItem = Item(
            name: NSLocalizedString("Rented only", comment: ""),
            type: .ownershipRented,
            isEnabled: true
        )

        static let ownerships: [Item] = [anyOwnershipItem, ownedOwnershipItem, rentedOwnershipItem]

        static var allProviders: Item {
            Item(
                name: NSLocalizedString("All providers", comment: ""),
                type: .allProviders,
                isEnabled: true
            )
        }

        static func < (lhs: Item, rhs: Item) -> Bool {
            let nameComparison = lhs.name.caseInsensitiveCompare(rhs.name)
            return nameComparison == .orderedAscending
        }
    }
}
