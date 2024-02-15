//
//  CustomListItemIdentifier.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum CustomListItemIdentifier: Hashable, CaseIterable {
    case name
    case locations

    enum CellIdentifier: String, CellIdentifierProtocol {
        case name
        case locations

        var cellClass: AnyClass {
            BasicCell.self
        }
    }

    var cellIdentifier: CellIdentifier {
        switch self {
        case .name:
            .name
        case .locations:
            .locations
        }
    }

    var isSelectable: Bool {
        switch self {
        case .name:
            false
        case .locations:
            true
        }
    }

    var text: String? {
        switch self {
        case .name:
            NSLocalizedString("NAME", tableName: "CustomLists", value: "Name", comment: "")
        case .locations:
            NSLocalizedString("ADD", tableName: "CustomLists", value: "Add locations", comment: "")
        }
    }
}
