//
//  RelayFilterDataSourceItem.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension RelayFilterSelection {
    // this is kept outside of DataSource, as it is needed in MullvadVPNtests, whereas DataSource most emphatically is not
    struct DataSourceItem: Hashable, Comparable {
        let name: String
        var description = ""
        let type: ItemType
        let isEnabled: Bool

        enum ItemType: Hashable {
            case ownershipAny, ownershipOwned, ownershipRented, allProviders, provider
        }

        static let anyOwnershipItem = DataSourceItem(
            name: NSLocalizedString("Any", comment: ""),
            type: .ownershipAny,
            isEnabled: true
        )

        static let ownedOwnershipItem = DataSourceItem(
            name: NSLocalizedString("Mullvad owned only", comment: ""),
            type: .ownershipOwned,
            isEnabled: true
        )

        static let rentedOwnershipItem = DataSourceItem(
            name: NSLocalizedString("Rented only", comment: ""),
            type: .ownershipRented,
            isEnabled: true
        )

        static let ownerships: [DataSourceItem] = [anyOwnershipItem, ownedOwnershipItem, rentedOwnershipItem]

        static var allProviders: DataSourceItem {
            DataSourceItem(
                name: NSLocalizedString("All providers", comment: ""),
                type: .allProviders,
                isEnabled: true
            )
        }

        static func < (lhs: DataSourceItem, rhs: DataSourceItem) -> Bool {
            let nameComparison = lhs.name.caseInsensitiveCompare(rhs.name)
            return nameComparison == .orderedAscending
        }
    }
}
