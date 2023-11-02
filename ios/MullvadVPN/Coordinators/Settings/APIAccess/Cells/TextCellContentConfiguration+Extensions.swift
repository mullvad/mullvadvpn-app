//
//  TextCellContentConfiguration+Extensions.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

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
                NSLocalizedString(
                    "REQUIRED_PLACEHOLDER",
                    tableName: "APIAccess",
                    value: "Required",
                    comment: ""
                )
            case .optional:
                NSLocalizedString(
                    "OPTIONAL_PLACEHOLDER",
                    tableName: "APIAccess",
                    value: "Optional",
                    comment: ""
                )
            }
        }
    }

    /// Set localized text placeholder using on the given placeholder type.
    /// - Parameter type: a placeholder type.
    mutating func setPlaceholder(type: PlaceholderType) {
        placeholder = type.localizedDescription
    }
}
