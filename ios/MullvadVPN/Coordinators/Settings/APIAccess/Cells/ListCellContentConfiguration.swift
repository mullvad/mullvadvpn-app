//
//  ListCellContentConfiguration.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-25.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content configuration presenting a label and switch control.
struct ListCellContentConfiguration: UIContentConfiguration, Equatable {
    struct TextProperties: Equatable {
        var font = UIFont.mullvadSmall
        var color = UIColor.Cell.titleTextColor
    }

    struct SecondaryTextProperties: Equatable {
        var font = UIFont.mullvadSmall
        var color = UIColor.Cell.detailTextColor.withAlphaComponent(0.6)
    }

    struct TertiaryTextProperties: Equatable {
        var font = UIFont.mullvadTiny
        var color = UIColor.Cell.titleTextColor.withAlphaComponent(0.6)
    }

    /// Primary text label.
    var text: String?
    var textProperties = TextProperties()

    /// Secondary (trailing) text label.
    var secondaryText: String?
    var secondaryTextProperties = SecondaryTextProperties()

    /// Tertiary (below primary) text label.
    var tertiaryText: String?
    var tertiaryTextProperties = TertiaryTextProperties()

    /// Content view layout margins.
    var directionalLayoutMargins = UIMetrics.SettingsCell.defaultLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return ListCellContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}
