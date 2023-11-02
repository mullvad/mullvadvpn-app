//
//  Binding.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import UIKit

extension CurrentValueSubject {
    /// Creates `UIAction` that automatically updates the value from text field.
    ///
    /// - Parameter keyPath: the key path to the field that should be updated.
    /// - Returns: an instance of `UIAction`.
    func bindTextAction(to keyPath: WritableKeyPath<Output, String>) -> UIAction {
        UIAction { action in
            guard let textField = action.sender as? UITextField else { return }

            self.value[keyPath: keyPath] = textField.text ?? ""
        }
    }

    /// Creates `UIAction` that automatically updates the value from input from a switch control.
    ///
    /// - Parameter keyPath: the key path to the field that should be updated.
    /// - Returns: an instance of `UIAction`.
    func bindSwitchAction(to keyPath: WritableKeyPath<Output, Bool>) -> UIAction {
        UIAction { action in
            guard let toggle = action.sender as? UISwitch else { return }

            self.value[keyPath: keyPath] = toggle.isOn
        }
    }
}
