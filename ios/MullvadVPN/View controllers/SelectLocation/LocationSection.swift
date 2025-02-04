//
//  LocationSection.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
enum LocationSection: String, Hashable, CustomStringConvertible, CaseIterable, CellIdentifierProtocol, Sendable {
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

    var cellClass: AnyClass {
        LocationCell.self
    }
}
