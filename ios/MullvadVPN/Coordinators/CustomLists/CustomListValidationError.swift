//
//  CustomListValidationError.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-16.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

enum CustomListFieldValidationError: LocalizedError, Hashable {
    case name(CustomRelayListError)

    var errorDescription: String? {
        switch self {
        case let .name(error):
            error.errorDescription
        }
    }
}

extension Collection<CustomListFieldValidationError> {
    var settingsFieldValidationErrors: [SettingsFieldValidationError] {
        map { SettingsFieldValidationError(errorDescription: $0.errorDescription) }
    }
}
