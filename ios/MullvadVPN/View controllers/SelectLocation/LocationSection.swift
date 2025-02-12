//
//  LocationSection.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
enum LocationSection: String, Hashable, CaseIterable, CellIdentifierProtocol, Sendable {
    case customLists
    case allLocations

    var header: String {
        switch self {
        case .customLists:
            return NSLocalizedString(
                "HEADER_SELECT_LOCATION_ADD_CUSTOM_LISTS",
                value: "Custom lists",
                comment: ""
            )
        case .allLocations:
            return NSLocalizedString(
                "HEADER_SELECT_LOCATION_ALL_LOCATIONS",
                value: "All locations",
                comment: ""
            )
        }
    }

    var footer: String {
        switch self {
        case .customLists:
            return ""
        case .allLocations:
            return NSLocalizedString(
                "FOOTER_SELECT_LOCATION_ALL_LOCATIONS",
                value: "No matching relays found, check your filter settings.",
                comment: ""
            )
        }
    }

    var cellClass: AnyClass {
        LocationCell.self
    }
}
