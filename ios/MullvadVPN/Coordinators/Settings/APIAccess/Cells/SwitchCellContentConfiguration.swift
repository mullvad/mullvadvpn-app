//
//  SwitchCellContentConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 09/11/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
