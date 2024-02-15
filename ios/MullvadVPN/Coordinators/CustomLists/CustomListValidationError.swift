//
//  CustomListValidationError.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-16.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum CustomListFieldValidationError: LocalizedError {
    case name

    var errorDescription: String {
        switch self {
        case .name:
            NSLocalizedString(
                "CUSTOM_LISTS_VALIDATION_ERROR_EMPTY_FIELD",
                tableName: "CutstomLists",
                value: "A custom list with this name exists, please choose a unique name.",
                comment: ""
            )
        }
    }
}
