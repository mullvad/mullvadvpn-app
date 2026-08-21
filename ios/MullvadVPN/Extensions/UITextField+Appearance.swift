// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import UIKit

extension UITextField {
    @MainActor
    struct SearchTextFieldAppearance {
        let placeholderTextColor: UIColor
        let textColor: UIColor
        let backgroundColor: UIColor
        let leftViewTintColor: UIColor

        static var active: SearchTextFieldAppearance {
            SearchTextFieldAppearance(
                placeholderTextColor: .SearchTextField.placeholderTextColor,
                textColor: .SearchTextField.textColor,
                backgroundColor: .SearchTextField.backgroundColor,
                leftViewTintColor: .SearchTextField.leftViewTintColor
            )
        }

        static var inactive: SearchTextFieldAppearance {
            SearchTextFieldAppearance(
                placeholderTextColor: .SearchTextField.inactivePlaceholderTextColor,
                textColor: .SearchTextField.inactiveTextColor,
                backgroundColor: .SearchTextField.inactiveBackgroundColor,
                leftViewTintColor: .SearchTextField.inactiveLeftViewTintColor
            )
        }

        func apply(to searchBar: UISearchBar) {
            searchBar.setImage(
                UIImage.Buttons.closeSmall.withTintColor(leftViewTintColor),
                for: .clear,
                state: .normal
            )

            apply(to: searchBar.searchTextField)
        }

        func apply(to textField: UITextField) {
            textField.leftView?.tintColor = leftViewTintColor
            textField.tintColor = textColor
            textField.textColor = textColor
            textField.backgroundColor = backgroundColor

            if let customTextField = textField as? CustomTextField {
                customTextField.placeholderTextColor = placeholderTextColor
            } else {
                textField.attributedPlaceholder = NSAttributedString(
                    string: textField.placeholder ?? "",
                    attributes: [.foregroundColor: placeholderTextColor]
                )
            }
        }
    }
}
