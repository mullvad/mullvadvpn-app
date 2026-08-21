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
