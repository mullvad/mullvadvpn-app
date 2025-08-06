//
//  CustomListItemIdentifier.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum CustomListItemIdentifier: Hashable, CaseIterable {
    case name
    case addLocations
    case editLocations
    case deleteList

    enum CellIdentifier: String, CellIdentifierProtocol {
        case name
        case locations
        case delete

        var cellClass: AnyClass {
            BasicCell.self
        }
    }

    var cellIdentifier: CellIdentifier {
        switch self {
        case .name:
            .name
        case .addLocations:
            .locations
        case .editLocations:
            .locations
        case .deleteList:
            .delete
        }
    }

    var text: String? {
        switch self {
        case .name:
            NSLocalizedString("NAME", value: "Name", comment: "")
        case .addLocations:
            NSLocalizedString("ADD", value: "Add locations", comment: "")
        case .editLocations:
            NSLocalizedString("EDIT", value: "Edit locations", comment: "")
        case .deleteList:
            NSLocalizedString("Delete", value: "Delete list", comment: "")
        }
    }

    static func fromFieldValidationErrors(_ errors: Set<CustomListFieldValidationError>) -> [CustomListItemIdentifier] {
        errors.compactMap { error in
            switch error {
            case .name:
                .name
            }
        }
    }
}
