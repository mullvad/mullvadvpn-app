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

/// Content configuration presenting a label and switch control.
struct SwitchCellContentConfiguration: UIContentConfiguration, Equatable {
    struct TextProperties: Equatable {
        var font = UIFont.mullvadSmall
        var color = UIColor.Cell.titleTextColor
    }

    var accessibilityIdentifier: AccessibilityIdentifier?

    /// Text label.
    var text: String?

    /// Whether the toggle is on or off.
    var isOn = false

    /// The action dispacthed on toggle change.
    var onChange: UIAction?

    /// Text label properties.
    var textProperties = TextProperties()

    /// Content view layout margins.
    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.defaultLayoutMargins

    /// Whether the toggle is enabled or disabled
    var isEnabled = true

    func makeContentView() -> UIView & UIContentView {
        return SwitchCellContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}
