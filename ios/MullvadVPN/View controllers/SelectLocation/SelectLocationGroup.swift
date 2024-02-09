//
//  SelectLocationSectionGroup.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
enum SelectLocationGroup: Hashable, CustomStringConvertible, CaseIterable {
    case customLists
    case allLocations

    var description: String {
        switch self {
        case .customLists:
            return NSLocalizedString(
                "SELECT_LOCATION_ADD_CUSTOM_LISTS",
                value: "Custom lists",
                comment: ""
            )
        case .allLocations:
            return NSLocalizedString(
                "SELECT_LOCATION_ALL_LOCATIONS",
                value: "All locations",
                comment: ""
            )
        }
    }

    var cell: Cell {
        Cell.locationCell
    }

    static var allCases: [SelectLocationGroup] {
        #if DEBUG
        return [.customLists, .allLocations]
        #else
        return [.allLocations]
        #endif
    }
}

extension SelectLocationGroup {
    enum Cell: String, CaseIterable {
        case locationCell

        var reusableViewClass: AnyClass {
            switch self {
            case .locationCell:
                return SelectLocationCell.self
            }
        }

        var reuseIdentifier: String {
            self.rawValue
        }
    }
}
