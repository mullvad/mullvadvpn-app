//
//  UITextField+Appearance.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
