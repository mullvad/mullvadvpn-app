//
//  UISearchBar+Appearance.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-04-04.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UISearchBar {
    struct SearchBarAppearance {
        let placeholderTextColor: UIColor
        let textColor: UIColor
        let backgroundColor: UIColor
        let leftViewTintColor: UIColor

        static var active: SearchBarAppearance {
            return SearchBarAppearance(
                placeholderTextColor: .SearchTextField.placeholderTextColor,
                textColor: .SearchTextField.textColor,
                backgroundColor: .SearchTextField.backgroundColor,
                leftViewTintColor: .SearchTextField.leftViewTintColor
            )
        }

        static var inactive: SearchBarAppearance {
            return SearchBarAppearance(
                placeholderTextColor: .SearchTextField.inactivePlaceholderTextColor,
                textColor: .SearchTextField.inactiveTextColor,
                backgroundColor: .SearchTextField.inactiveBackgroundColor,
                leftViewTintColor: .SearchTextField.inactiveLeftViewTintColor
            )
        }

        func apply(to searchBar: UISearchBar) {
            let textField = searchBar.searchTextField

            textField.leftView?.tintColor = leftViewTintColor
            textField.tintColor = textColor
            textField.textColor = textColor
            textField.backgroundColor = backgroundColor
            textField.attributedPlaceholder = NSAttributedString(
                string: searchBar.placeholder ?? "",
                attributes: [
                    .foregroundColor: placeholderTextColor,
                ]
            )
        }
    }
}
