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
import Routing

struct AlertMetadata {
    let presentation: AlertPresentation
    let context: Presenting
}

struct AlertAction {
    let title: String
    let style: AlertActionStyle
    var accessibilityId: AccessibilityIdentifier?
    var handler: (() -> Void)?
    var interactiveHandler: ((AlertViewController, AppButton) -> Void)?
}

struct AlertPresentation: Identifiable, CustomDebugStringConvertible {
    let id: String

    var accessibilityIdentifier: AccessibilityIdentifier?
    var header: String?
    var icon: AlertIcon?
    var title: String?
    var message: String?
    var attributedMessage: NSAttributedString?
    let buttons: [AlertAction]

    var debugDescription: String {
        return id
    }
}

extension AlertPresentation: Equatable, Hashable {
    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: AlertPresentation, rhs: AlertPresentation) -> Bool {
        return lhs.id == rhs.id
    }
}
