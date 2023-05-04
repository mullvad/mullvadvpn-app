//
//  UISearchBar+Appearance.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-04.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UITextField {
    struct SearchTextFieldAppearance {
        let placeholderTextColor: UIColor
        let textColor: UIColor
        let backgroundColor: UIColor
        let leftViewTintColor: UIColor

        static var active: SearchTextFieldAppearance {
            return SearchTextFieldAppearance(
                placeholderTextColor: .SearchTextField.placeholderTextColor,
                textColor: .SearchTextField.textColor,
                backgroundColor: .SearchTextField.backgroundColor,
                leftViewTintColor: .SearchTextField.leftViewTintColor
            )
        }

        static var inactive: SearchTextFieldAppearance {
            return SearchTextFieldAppearance(
                placeholderTextColor: .SearchTextField.inactivePlaceholderTextColor,
                textColor: .SearchTextField.inactiveTextColor,
                backgroundColor: .SearchTextField.inactiveBackgroundColor,
                leftViewTintColor: .SearchTextField.inactiveLeftViewTintColor
            )
        }

        func apply(to searchBar: UITextField) {
            let textField = searchBar

            textField.leftView?.tintColor = leftViewTintColor
            textField.tintColor = textColor
            textField.textColor = textColor
            textField.backgroundColor = backgroundColor

            if let customTextField = searchBar as? CustomTextField {
                customTextField.placeholderTextColor = placeholderTextColor
            } else {
                textField.attributedPlaceholder = NSAttributedString(
                    string: searchBar.placeholder ?? "",
                    attributes: [.foregroundColor: placeholderTextColor]
                )
            }
        }
    }
}
