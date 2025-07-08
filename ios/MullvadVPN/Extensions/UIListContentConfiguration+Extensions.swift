//
//  UIListContentConfiguration+Extensions.swift
//  MullvadVPN
//
//  Created by pronebird on 09/11/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIListContentConfiguration {
    /// Returns cell configured with default text attribute used in Mullvad UI.
    static func mullvadCell(tableStyle: UITableView.Style, isEnabled: Bool = true) -> UIListContentConfiguration {
        var configuration = cell()
        configuration.textProperties.font = .mullvadSmall
        configuration.textProperties.adjustsFontForContentSizeCategory = true
        configuration.textProperties.color = .Cell.titleTextColor.withAlphaComponent(isEnabled ? 1 : 0.8)
        configuration.secondaryTextProperties.font = .mullvadSmall
        configuration.secondaryTextProperties.adjustsFontForContentSizeCategory = true

        applyMargins(to: &configuration, tableStyle: tableStyle)

        return configuration
    }

    /// Returns value cell configured with default text attribute used in Mullvad UI.
    static func mullvadValueCell(tableStyle: UITableView.Style, isEnabled: Bool = true) -> UIListContentConfiguration {
        var configuration = valueCell()
        configuration.textProperties.font = .mullvadSmall
        configuration.textProperties.adjustsFontForContentSizeCategory = true
        configuration.textProperties.color = .Cell.titleTextColor.withAlphaComponent(isEnabled ? 1 : 0.8)
        configuration.secondaryTextProperties.color = .Cell.detailTextColor.withAlphaComponent(0.8)
        configuration.secondaryTextProperties.font = .mullvadSmall
        configuration.secondaryTextProperties.adjustsFontForContentSizeCategory = true

        applyMargins(to: &configuration, tableStyle: tableStyle)

        return configuration
    }

    /// Returns grouped header configured with default text attribute used in Mullvad UI.
    static func mullvadGroupedHeader(tableStyle: UITableView.Style) -> UIListContentConfiguration {
        var configuration = groupedHeader()
        configuration.textProperties.color = .TableSection.headerTextColor
        configuration.textProperties.font = .mullvadTiny
        configuration.textProperties.adjustsFontForContentSizeCategory = true
        configuration.textProperties.numberOfLines = 0

        applyMargins(to: &configuration, tableStyle: tableStyle)

        return configuration
    }

    /// Returns grouped footer configured with default text attribute used in Mullvad UI.
    static func mullvadGroupedFooter(tableStyle: UITableView.Style) -> UIListContentConfiguration {
        var configuration = groupedFooter()
        configuration.textProperties.color = .TableSection.footerTextColor
        configuration.textProperties.font = .mullvadMini
        configuration.textProperties.adjustsFontForContentSizeCategory = true
        configuration.textProperties.numberOfLines = 0
        applyMargins(to: &configuration, tableStyle: tableStyle)

        return configuration
    }

    private static func applyMargins(
        to configuration: inout UIListContentConfiguration,
        tableStyle: UITableView.Style
    ) {
        configuration.axesPreservingSuperviewLayoutMargins = .vertical
        configuration.directionalLayoutMargins = tableStyle.directionalLayoutMarginsForCell
    }
}

extension UITableView.Style {
    var directionalLayoutMarginsForCell: NSDirectionalEdgeInsets {
        switch self {
        case .plain, .grouped:
            UIMetrics.SettingsCell.apiAccessLayoutMargins
        case .insetGrouped:
            UIMetrics.SettingsCell.apiAccessInsetLayoutMargins
        @unknown default:
            UIMetrics.SettingsCell.apiAccessLayoutMargins
        }
    }
}
