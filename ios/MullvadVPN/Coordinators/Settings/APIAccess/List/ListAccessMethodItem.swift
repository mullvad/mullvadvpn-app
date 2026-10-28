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

enum AccessMethodStatus: Equatable, Hashable {
    case on
    case off
    case inUse
    case warning(String)
    case error(String)

    var description: String {
        switch self {
        case .off:
            NSLocalizedString("Off", comment: "")
        case .inUse:
            NSLocalizedString("In use", comment: "")
        case .warning(let message), .error(let message):
            NSLocalizedString(message, comment: "")
        case .on:
            ""
        }
    }
}

/// A concrete implementation of an API access list item.
struct ListAccessMethodItem: Hashable, Identifiable, Equatable {
    /// The unique ID.
    let id: UUID

    /// The localized name.
    let name: String

    /// The detailed information displayed alongside.
    let detail: String?

    /// Whether method is enabled or not.
    let isEnabled: Bool

    /// Indicates whether the method is enabled or currently in use,
    /// or whether it is unsupported.
    let status: AccessMethodStatus

    var accessibilityIdentifier: AccessibilityIdentifier? {
        switch id {
        case AccessMethodRepository.directId:
            AccessibilityIdentifier.accessMethodDirectCell
        case AccessMethodRepository.bridgeId:
            AccessibilityIdentifier.accessMethodBridgesCell
        case AccessMethodRepository.encryptedDNSId:
            AccessibilityIdentifier.accessMethodEncryptedDNSCell
        case AccessMethodRepository.domainFrontingId:
            AccessibilityIdentifier.accessMethodDomainFrontingCell
        default:
            nil
        }
    }
}
