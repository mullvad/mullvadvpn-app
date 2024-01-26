//
//  ListCellContentConfiguration.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-25.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Content configuration presenting a label and switch control.
struct ListCellContentConfiguration: UIContentConfiguration, Equatable {
    struct TextProperties: Equatable {
        var font = UIFont.systemFont(ofSize: 17)
        var color = UIColor.Cell.titleTextColor
    }

    struct SecondaryTextProperties: Equatable {
        var font = UIFont.systemFont(ofSize: 17)
        var color = UIColor.Cell.detailTextColor.withAlphaComponent(0.8)
    }

    struct TertiaryTextProperties: Equatable {
        var font = UIFont.systemFont(ofSize: 15)
        var color = UIColor.Cell.titleTextColor.withAlphaComponent(0.6)
    }

    /// Primary text label.
    var text: String?
    let textProperties = TextProperties()

    /// Secondary (trailing) text label.
    var secondaryText: String?
    let secondaryTextProperties = SecondaryTextProperties()

    /// Tertiary (below primary) text label.
    var tertiaryText: String?
    let tertiaryTextProperties = TertiaryTextProperties()

    /// Content view layout margins.
    var directionalLayoutMargins: NSDirectionalEdgeInsets = UIMetrics.SettingsCell.apiAccessInsetLayoutMargins

    func makeContentView() -> UIView & UIContentView {
        return ListCellContentView(configuration: self)
    }

    func updated(for state: UIConfigurationState) -> Self {
        return self
    }
}
