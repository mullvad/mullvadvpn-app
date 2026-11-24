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

extension TextCellContentConfiguration.TextFieldProperties {
    /// Returns text field properties configured with automatic resign on return key and "done" return key.
    static func withAutoResignAndDoneReturnKey() -> Self {
        .init(resignOnReturn: true, returnKey: .done)
    }

    /// Returns text field properties configured with automatic resign on return key and "done" return key and all auto-correction and smart features disabled.
    static func withSmartFeaturesDisabled() -> Self {
        withAutoResignAndDoneReturnKey().disabling(features: .all)
    }
}

extension TextCellContentConfiguration {
    /// Type of placeholder to set on the text field.
    enum PlaceholderType {
        case required, optional

        var localizedDescription: String {
            switch self {
            case .required:
                NSLocalizedString("Required", comment: "")
            case .optional:
                NSLocalizedString("Optional", comment: "")
            }
        }
    }

    /// Set localized text placeholder using on the given placeholder type.
    /// - Parameter type: a placeholder type.
    mutating func setPlaceholder(type: PlaceholderType) {
        placeholder = type.localizedDescription
    }
}
