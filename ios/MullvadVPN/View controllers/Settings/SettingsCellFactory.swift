//
//  SettingsCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

protocol SettingsCellEventHandler {
    func showInfo(for button: SettingsInfoButtonItem)
}

@MainActor
final class SettingsCellFactory: @preconcurrency CellFactoryProtocol, Sendable {
    let tableView: UITableView
    var delegate: SettingsCellEventHandler?
    var viewModel: SettingsViewModel
    private let interactor: SettingsInteractor

    init(tableView: UITableView, interactor: SettingsInteractor) {
        self.tableView = tableView
        self.interactor = interactor

        viewModel = SettingsViewModel(from: interactor.tunnelSettings)
    }

    func makeCell(for item: SettingsDataSource.Item, indexPath: IndexPath) -> UITableViewCell {
        let cell: UITableViewCell

        cell = tableView
            .dequeueReusableCell(
                withIdentifier: item.reuseIdentifier.rawValue
            ) ?? SettingsCell(
                style: item.reuseIdentifier.cellStyle,
                reuseIdentifier: item.reuseIdentifier.rawValue
            )

        // Configure the cell with the common logic
        configureCell(cell, item: item, indexPath: indexPath)

        return cell
    }

    // swiftlint:disable:next function_body_length
    func configureCell(_ cell: UITableViewCell, item: SettingsDataSource.Item, indexPath: IndexPath) {
        switch item {
        case .vpnSettings:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "VPN_SETTINGS_CELL_LABEL",
                tableName: "Settings",
                value: "VPN settings",
                comment: ""
            )
            cell.detailTitleLabel.text = nil
            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron

        case .changelog:
            guard let cell = cell as? SettingsCell else { return }
            cell.titleLabel.text = NSLocalizedString(
                "APP_VERSION_CELL_LABEL",
                tableName: "Settings",
                value: "What's new",
                comment: ""
            )
            cell.detailTitleLabel.text = Bundle.main.productVersion
            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron

        case .problemReport:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "REPORT_PROBLEM_CELL_LABEL",
                tableName: "Settings",
                value: "Report a problem",
                comment: ""
            )
            cell.detailTitleLabel.text = nil
            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron

        case .faq:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "FAQ_AND_GUIDES_CELL_LABEL",
                tableName: "Settings",
                value: "FAQs & Guides",
                comment: ""
            )
            cell.detailTitleLabel.text = nil
            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .externalLink

        case .apiAccess:
            guard let cell = cell as? SettingsCell else { return }
            cell.titleLabel.text = NSLocalizedString(
                "API_ACCESS_CELL_LABEL",
                tableName: "Settings",
                value: "API access",
                comment: ""
            )
            cell.detailTitleLabel.text = nil
            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron

        case .daita:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "DAITA_CELL_LABEL",
                tableName: "Settings",
                value: "DAITA",
                comment: ""
            )

            cell.detailTitleLabel.text = NSLocalizedString(
                "DAITA_CELL_DETAIL_LABEL",
                tableName: "Settings",
                value: viewModel.daitaSettings.daitaState.isEnabled ? "On" : "Off",
                comment: ""
            )

            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron

        case .multihop:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "MULTIHOP_CELL_LABEL",
                tableName: "Settings",
                value: "Multihop",
                comment: ""
            )

            cell.detailTitleLabel.text = NSLocalizedString(
                "MULTIHOP_CELL_DETAIL_LABEL",
                tableName: "Settings",
                value: viewModel.multihopState.isEnabled ? "On" : "Off",
                comment: ""
            )

            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron
        }
    }
}
