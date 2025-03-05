//
//  RelayFilterDataSourceCells.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

// MARK: - Data Models

extension RelayFilterDataSource {
    enum Section: CaseIterable { case ownership, providers }

    struct Item: Hashable, Comparable {
        let name: String
        let type: ItemType
        let isEnabled: Bool

        enum ItemType: Hashable {
            case ownershipAny, ownershipOwned, ownershipRented, allProviders, provider(name: String)
        }

        static var ownerships: [Item] {
            [
                Item(name: "Any", type: .ownershipAny, isEnabled: true),
                Item(name: "Owned", type: .ownershipOwned, isEnabled: true),
                Item(name: "Rented", type: .ownershipRented, isEnabled: true),
            ]
        }

        static var allProviders: Item {
            Item(name: "All Providers", type: .allProviders, isEnabled: true)
        }

        static func provider(name: String, isEnabled: Bool) -> Item {
            Item(name: name, type: .provider(name: name), isEnabled: isEnabled)
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
