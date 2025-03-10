//
//  RelayFilterDataSourceItem.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension RelayFilterDataSource {
    enum Section: CaseIterable { case ownership, providers }

    struct Item: Hashable, Comparable {
        let name: String
        var description = ""
        let type: ItemType
        let isEnabled: Bool

        enum ItemType: Hashable {
            case ownershipAny, ownershipOwned, ownershipRented, allProviders, provider
        }

        static var ownerships: [Item] {
            [
                Item(name: NSLocalizedString(
                    "RELAY_FILTER_ANY_LABEL",
                    tableName: "RelayFilter",
                    value: "Any",
                    comment: ""
                ), type: .ownershipAny, isEnabled: true),

                Item(name: NSLocalizedString(
                    "RELAY_FILTER_OWNED_LABEL",
                    tableName: "RelayFilter",
                    value: "Owned",
                    comment: ""
                ), type: .ownershipOwned, isEnabled: true),
                Item(name: NSLocalizedString(
                    "RELAY_FILTER_RENTED_LABEL",
                    tableName: "RelayFilter",
                    value: "Rented",
                    comment: ""
                ), type: .ownershipRented, isEnabled: true),
            ]
        }

        static var allProviders: Item {
            Item(name: NSLocalizedString(
                "RELAY_FILTER_ALL_PROVIDERS_LABEL",
                tableName: "RelayFilter",
                value: "All Providers",
                comment: ""
            ), type: .allProviders, isEnabled: true)
        }

        static func < (lhs: Item, rhs: Item) -> Bool {
            let nameComparison = lhs.name.caseInsensitiveCompare(rhs.name)
            return nameComparison == .orderedAscending
        }
    }
}

// MARK: - Cell Identifiers

extension RelayFilterDataSource {
    enum CellReuseIdentifiers: String, CaseIterable {
        case ownershipCell, providerCell

        var reusableViewClass: AnyClass {
            switch self {
            case .ownershipCell: return SelectableSettingsCell.self
            case .providerCell: return CheckableSettingsCell.self
            }
        }
    }

    enum HeaderFooterReuseIdentifiers: String, CaseIterable {
        case section

        var reusableViewClass: AnyClass { SettingsHeaderView.self }
    }
}
