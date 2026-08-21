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
            NSLocalizedString("Name", comment: "")
        case .addLocations, .editLocations:
            NSLocalizedString("Locations", comment: "")
        case .deleteList:
            NSLocalizedString("Delete list", comment: "")
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
