//
//  ButtonCellConfiguration.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
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

    /// The button content edge insets.
    var directionalContentEdgeInsets: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.insetLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return ButtonCellContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}

extension ButtonCellContentConfiguration {
    struct TextProperties: Equatable {
        var font = UIFont.systemFont(ofSize: 17)
        var color = UIColor.Cell.titleTextColor
    }
}
