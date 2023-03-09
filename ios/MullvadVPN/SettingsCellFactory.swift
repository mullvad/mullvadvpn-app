//
//  SettingsCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class SettingsCellFactory: CellFactoryProtocol {
    var tableView: UITableView?
    var interactor: SettingsInteractor?

    func makeCell(for item: SettingsDataSource.Item, indexPath: IndexPath) -> UITableViewCell {
        switch item {
        case .account:
            let cell = tableView?.dequeueReusableCell(
                withIdentifier: SettingsDataSource.CellReuseIdentifiers.accountCell.rawValue,
                for: indexPath
            ) as! SettingsAccountCell
            cell.titleLabel.text = NSLocalizedString(
                "ACCOUNT_CELL_LABEL",
                tableName: "Settings",
                value: "Account",
                comment: ""
            )
            cell.accountExpiryDate = interactor?.deviceState.accountData?.expiry
            cell.accessibilityIdentifier = "AccountCell"
            cell.disclosureType = .chevron

            return cell

        case .preferences:
            let cell = tableView?.dequeueReusableCell(
                withIdentifier: SettingsDataSource.CellReuseIdentifiers.basicCell.rawValue,
                for: indexPath
            ) as! SettingsCell
            cell.titleLabel.text = NSLocalizedString(
                "PREFERENCES_CELL_LABEL",
                tableName: "Settings",
                value: "Preferences",
                comment: ""
            )
            cell.detailTitleLabel.text = nil
            cell.accessibilityIdentifier = "PreferencesCell"
            cell.disclosureType = .chevron

            return cell

        case .version:
            let cell = tableView?.dequeueReusableCell(
                withIdentifier: SettingsDataSource.CellReuseIdentifiers.basicCell.rawValue,
                for: indexPath
            ) as! SettingsCell
            cell.titleLabel.text = NSLocalizedString(
                "APP_VERSION_CELL_LABEL",
                tableName: "Settings",
                value: "App version",
                comment: ""
            )
            cell.detailTitleLabel.text = Bundle.main.productVersion
            cell.accessibilityIdentifier = nil
            cell.disclosureType = .none

            return cell

        case .problemReport:
            let cell = tableView?.dequeueReusableCell(
                withIdentifier: SettingsDataSource.CellReuseIdentifiers.basicCell.rawValue,
                for: indexPath
            ) as! SettingsCell
            cell.titleLabel.text = NSLocalizedString(
                "REPORT_PROBLEM_CELL_LABEL",
                tableName: "Settings",
                value: "Report a problem",
                comment: ""
            )
            cell.detailTitleLabel.text = nil
            cell.accessibilityIdentifier = nil
            cell.disclosureType = .chevron

            return cell

        case .faq:
            let cell = tableView?.dequeueReusableCell(
                withIdentifier: SettingsDataSource.CellReuseIdentifiers.basicCell.rawValue,
                for: indexPath
            ) as! SettingsCell
            cell.titleLabel.text = NSLocalizedString(
                "FAQ_AND_GUIDES_CELL_LABEL",
                tableName: "Settings",
                value: "FAQ & Guides",
                comment: ""
            )
            cell.detailTitleLabel.text = nil
            cell.accessibilityIdentifier = nil
            cell.disclosureType = .externalLink

            return cell
        }
    }
}
