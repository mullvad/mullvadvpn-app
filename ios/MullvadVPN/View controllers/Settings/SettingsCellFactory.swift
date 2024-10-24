//
//  SettingsCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

protocol SettingsCellEventHandler {
    func showInfo(for button: SettingsInfoButtonItem)
    func switchDaitaState(_ settings: DAITASettings)
    func switchDaitaDirectOnlyState(_ settings: DAITASettings)
}

final class SettingsCellFactory: CellFactoryProtocol {
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
        let cell = tableView.dequeueReusableCell(withIdentifier: item.reuseIdentifier.rawValue, for: indexPath)

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
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.disclosureType = .chevron

        case .version:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "APP_VERSION_CELL_LABEL",
                tableName: "Settings",
                value: "App version",
                comment: ""
            )
            cell.detailTitleLabel.text = Bundle.main.productVersion
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.disclosureType = .none

        case .problemReport:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "REPORT_PROBLEM_CELL_LABEL",
                tableName: "Settings",
                value: "Report a problem",
                comment: ""
            )
            cell.detailTitleLabel.text = nil
            cell.accessibilityIdentifier = item.accessibilityIdentifier
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
            cell.accessibilityIdentifier = item.accessibilityIdentifier
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
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.disclosureType = .chevron

        case .daita:
            guard let cell = cell as? SettingsSwitchCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "DAITA_LABEL",
                tableName: "Settings",
                value: "DAITA",
                comment: ""
            )
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.setOn(viewModel.daitaSettings.daitaState.isEnabled, animated: false)

            cell.infoButtonHandler = { [weak self] in
                self?.delegate?.showInfo(for: .daita)
            }

            cell.action = { [weak self] isEnabled in
                guard let self else { return }

                let state: DAITAState = isEnabled ? .on : .off
                delegate?.switchDaitaState(DAITASettings(
                    daitaState: state,
                    directOnlyState: viewModel.daitaSettings.directOnlyState
                ))
            }

        case .daitaDirectOnly:
            guard let cell = cell as? SettingsSwitchCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "DAITA_DIRECT_ONLY_LABEL",
                tableName: "Settings",
                value: "Direct only",
                comment: ""
            )
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.setOn(viewModel.daitaSettings.directOnlyState.isEnabled, animated: false)
            cell.setSwitchEnabled(viewModel.daitaSettings.daitaState.isEnabled)

            cell.infoButtonHandler = { [weak self] in
                self?.delegate?.showInfo(for: .daitaDirectOnly)
            }

            cell.action = { [weak self] isEnabled in
                guard let self else { return }

                let state: DirectOnlyState = isEnabled ? .on : .off
                delegate?.switchDaitaDirectOnlyState(DAITASettings(
                    daitaState: viewModel.daitaSettings.daitaState,
                    directOnlyState: state
                ))
            }
        }
    }
}
