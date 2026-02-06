//
//  ButtonCellConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// The content configuration for cells that contain the full-width button.
struct ButtonCellContentConfiguration: UIContentConfiguration, Equatable {
    /// Button label.
    var text: String?

    /// Button style.
    var style: AppButton.Style = .default

    /// Indicates whether button is enabled.
    var isEnabled = true

    /// Primary action for button.
    var primaryAction: UIAction?

    /// Style for displayed text.
    var textProperties = TextProperties()

    /// The button content edge insets.
    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.defaultLayoutMargins

    // Accessibility identifier.
    var accessibilityIdentifier: AccessibilityIdentifier?

    func makeContentView() -> UIView & UIContentView {
        return ButtonCellContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}

extension ButtonCellContentConfiguration {
    struct TextProperties: Equatable {
        var font = UIFont.mullvadSmallSemiBold
        var color = UIColor.Cell.titleTextColor
    }
}
