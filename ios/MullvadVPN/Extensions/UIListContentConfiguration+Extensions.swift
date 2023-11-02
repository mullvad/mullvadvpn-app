//
//  UIListContentConfiguration+Extensions.swift
//  MullvadVPN
//
//  Created by pronebird on 09/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIListContentConfiguration {
    /// Returns cell configured with default text attribute used in Mullvad UI.
    static func mullvadCell(tableStyle: UITableView.Style) -> UIListContentConfiguration {
        var configuration = cell()
        configuration.textProperties.font = UIFont.systemFont(ofSize: 17)
        configuration.textProperties.color = UIColor.Cell.titleTextColor
        configuration.directionalLayoutMargins = tableStyle.directionalLayoutMarginsForCell
        return configuration
    }

    /// Returns value cell configured with default text attribute used in Mullvad UI.
    static func mullvadValueCell(tableStyle: UITableView.Style) -> UIListContentConfiguration {
        var configuration = valueCell()
        configuration.textProperties.font = UIFont.systemFont(ofSize: 17)
        configuration.textProperties.color = UIColor.Cell.titleTextColor
        configuration.secondaryTextProperties.color = UIColor.Cell.detailTextColor
        configuration.secondaryTextProperties.font = UIFont.systemFont(ofSize: 17)
        configuration.directionalLayoutMargins = tableStyle.directionalLayoutMarginsForCell
        return configuration
    }

    /// Returns grouped header configured with default text attribute used in Mullvad UI.
    static func mullvadGroupedHeader() -> UIListContentConfiguration {
        var configuration = groupedHeader()
        configuration.textProperties.color = UIColor.TableSection.headerTextColor
        configuration.textProperties.font = UIFont.systemFont(ofSize: 17)
        return configuration
    }

    /// Returns grouped footer configured with default text attribute used in Mullvad UI.
    static func mullvadGroupedFooter() -> UIListContentConfiguration {
        var configuration = groupedFooter()
        configuration.textProperties.color = UIColor.TableSection.footerTextColor
        configuration.textProperties.font = UIFont.systemFont(ofSize: 14)
        return configuration
    }
}

extension UITableView.Style {
    var directionalLayoutMarginsForCell: NSDirectionalEdgeInsets {
        switch self {
        case .plain, .grouped:
            UIMetrics.SettingsCell.layoutMargins
        case .insetGrouped:
            UIMetrics.SettingsCell.insetLayoutMargins
        @unknown default:
            UIMetrics.SettingsCell.layoutMargins
        }
    }
}
