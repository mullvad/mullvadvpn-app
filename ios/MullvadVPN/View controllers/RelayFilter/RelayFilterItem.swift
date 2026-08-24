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

class RelayFilterItem: @unchecked Sendable {
    enum ItemType: Hashable {
        case ownershipAny, ownershipOwned, ownershipRented, allProviders, provider

        var accessibilityIdentifier: AccessibilityIdentifier {
            switch self {
            case .ownershipAny:
                .ownershipAnyCell
            case .ownershipOwned:
                .ownershipMullvadOwnedCell
            case .ownershipRented:
                .ownershipRentedCell
            case .allProviders:
                .relayFilterProviderCell
            case .provider:
                .relayFilterProviderCell
            }
        }
    }

    let name: String
    let type: ItemType
    var isSelected: Bool

    init(name: String, type: ItemType, isSelected: Bool) {
        self.name = name
        self.type = type
        self.isSelected = isSelected
    }

    static func anyOwnershipItem(isSelected: Bool = false) -> RelayFilterItem {
        RelayFilterItem(
            name: NSLocalizedString("Any", comment: ""),
            type: .ownershipAny,
            isSelected: isSelected
        )
    }

    static func ownedOwnershipItem(isSelected: Bool = false) -> RelayFilterItem {
        RelayFilterItem(
            name: NSLocalizedString("Mullvad owned only", comment: ""),
            type: .ownershipOwned,
            isSelected: isSelected
        )
    }

    static func rentedOwnershipItem(isSelected: Bool = false) -> RelayFilterItem {
        RelayFilterItem(
            name: NSLocalizedString("Rented only", comment: ""),
            type: .ownershipRented,
            isSelected: isSelected
        )
    }

    static func allProviders(isSelected: Bool = false) -> RelayFilterItem {
        RelayFilterItem(
            name: NSLocalizedString("All providers", comment: ""),
            type: .allProviders,
            isSelected: isSelected
        )
    }
}

extension RelayFilterItem: Equatable {
    static func == (lhs: RelayFilterItem, rhs: RelayFilterItem) -> Bool {
        lhs.name == rhs.name
    }
}

extension RelayFilterItem: Comparable {
    static func < (lhs: RelayFilterItem, rhs: RelayFilterItem) -> Bool {
        let nameComparison = lhs.name.caseInsensitiveCompare(rhs.name)
        return nameComparison == .orderedAscending
    }
}

extension RelayFilterItem: Hashable {
    func hash(into hasher: inout Hasher) {
        hasher.combine(name)
    }
}
