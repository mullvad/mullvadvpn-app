// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import UIKit

extension CurrentValueSubject {
    /// Creates `UIAction` that automatically updates the value from text field.
    ///
    /// - Parameter keyPath: the key path to the field that should be updated.
    /// - Returns: an instance of `UIAction`.
    @MainActor func bindTextAction(to keyPath: WritableKeyPath<Output, String>) -> UIAction {
        UIAction { action in
            guard let textField = action.sender as? UITextField else { return }

            self.value[keyPath: keyPath] = textField.text ?? ""
        }
    }

    /// Creates `UIAction` that automatically updates the value from input from a switch control.
    ///
    /// - Parameter keyPath: the key path to the field that should be updated.
    /// - Returns: an instance of `UIAction`.
    @MainActor func bindSwitchAction(to keyPath: WritableKeyPath<Output, Bool>) -> UIAction {
        UIAction { action in
            guard let toggle = action.sender as? UISwitch else { return }

            self.value[keyPath: keyPath] = toggle.isOn
        }
    }
}
