//
//  SettingsCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

struct SettingsCellFactory: CellFactoryProtocol {
    internal let tableView: UITableView
    private let interactor: SettingsInteractor

    init(tableView: UITableView, interactor: SettingsInteractor) {
        self.tableView = tableView
        self.interactor = interactor
    }

    func makeCell(for item: SettingsDataSource.Item, indexPath: IndexPath) -> UITableViewCell {
        var cell = UITableViewCell()

        switch item {
        case .account:
            cell = tableView.dequeueReusableCell(
                withIdentifier: SettingsDataSource.CellReuseIdentifiers.accountCell.rawValue,
                for: indexPath
            )
        default:
            cell = tableView.dequeueReusableCell(
                withIdentifier: SettingsDataSource.CellReuseIdentifiers.basicCell.rawValue,
                for: indexPath
            )
        }

        configureCell(cell, item: item, indexPath: indexPath)

        return cell
    }

    func configureCell(
        _ cell: UITableViewCell,
        item: SettingsDataSource.Item,
        indexPath: IndexPath
    ) {
        switch item {
        case .account:
            if let cell = cell as? SettingsAccountCell {
                cell.titleLabel.text = NSLocalizedString(
                    "ACCOUNT_CELL_LABEL",
                    tableName: "Settings",
                    value: "Account",
                    comment: ""
                )
                cell.accountExpiryDate = interactor.deviceState.accountData?.expiry
                cell.accessibilityIdentifier = "AccountCell"
                cell.disclosureType = .chevron
            }

        case .preferences:
            if let cell = cell as? SettingsCell {
                cell.titleLabel.text = NSLocalizedString(
                    "PREFERENCES_CELL_LABEL",
                    tableName: "Settings",
                    value: "Preferences",
                    comment: ""
                )
                cell.detailTitleLabel.text = nil
                cell.accessibilityIdentifier = "PreferencesCell"
                cell.disclosureType = .chevron
            }

        case .version:
            if let cell = cell as? SettingsCell {
                cell.titleLabel.text = NSLocalizedString(
                    "APP_VERSION_CELL_LABEL",
                    tableName: "Settings",
                    value: "App version",
                    comment: ""
                )
                cell.detailTitleLabel.text = Bundle.main.productVersion
                cell.accessibilityIdentifier = nil
                cell.disclosureType = .none
            }

        case .problemReport:
            if let cell = cell as? SettingsCell {
                cell.titleLabel.text = NSLocalizedString(
                    "REPORT_PROBLEM_CELL_LABEL",
                    tableName: "Settings",
                    value: "Report a problem",
                    comment: ""
                )
                cell.detailTitleLabel.text = nil
                cell.accessibilityIdentifier = nil
                cell.disclosureType = .chevron
            }

        case .faq:
            if let cell = cell as? SettingsCell {
                cell.titleLabel.text = NSLocalizedString(
                    "FAQ_AND_GUIDES_CELL_LABEL",
                    tableName: "Settings",
                    value: "FAQ & Guides",
                    comment: ""
                )
                cell.detailTitleLabel.text = nil
                cell.accessibilityIdentifier = nil
                cell.disclosureType = .externalLink
            }
        }
    }
}
