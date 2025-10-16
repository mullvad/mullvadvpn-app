//
//  SettingsCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

@MainActor
final class SettingsCellFactory: @preconcurrency CellFactoryProtocol {
    let tableView: UITableView
    var viewModel: SettingsViewModel
    private let interactor: SettingsInteractor
    private var contentSizeCategory = UIApplication.shared.preferredContentSizeCategory

    init(tableView: UITableView, interactor: SettingsInteractor) {
        self.tableView = tableView
        self.interactor = interactor

        viewModel = SettingsViewModel(from: interactor.tunnelSettings)

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(preferredContentSizeChanged(_:)),
            name: UIContentSizeCategory.didChangeNotification,
            object: nil
        )
    }

    func makeCell(for item: SettingsDataSource.Item, indexPath: IndexPath) -> UITableViewCell {
        let cell: UITableViewCell

        cell =
            tableView
            .dequeueReusableCell(
                withIdentifier: item.reuseIdentifier.rawValue
            )
            ?? SettingsCell(
                style: contentSizeCategory.isLarge ? .subtitle : item.reuseIdentifier.cellStyle,
                reuseIdentifier: item.reuseIdentifier.rawValue
            )

        // Configure the cell with the common logic
        configureCell(cell, item: item, indexPath: indexPath)

        return cell
    }

    func configureCell(_ cell: UITableViewCell, item: SettingsDataSource.Item, indexPath: IndexPath) {
        switch item {
        case .vpnSettings:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString("VPN settings", comment: "")
            cell.detailTitleLabel.text = nil
            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron

        case .changelog:
            guard let cell = cell as? SettingsCell else { return }
            cell.titleLabel.text = NSLocalizedString("What’s new", comment: "")
            cell.detailTitleLabel.text = Bundle.main.productVersion
            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron

        case .problemReport:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString("Report a problem", comment: "")
            cell.detailTitleLabel.text = nil
            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron

        case .faq:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString("FAQs & Guides", comment: "")
            cell.detailTitleLabel.text = nil
            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .externalLink

        case .apiAccess:
            guard let cell = cell as? SettingsCell else { return }
            cell.titleLabel.text = NSLocalizedString("API access", comment: "")
            cell.detailTitleLabel.text = nil
            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron

        case .daita:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString("DAITA", comment: "")

            cell.detailTitleLabel.text =
                viewModel.daitaSettings.daitaState.isEnabled
                ? NSLocalizedString("On", comment: "")
                : NSLocalizedString("Off", comment: "")

            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron

        case .multihop:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString("Multihop", comment: "")

            cell.detailTitleLabel.text =
                viewModel.multihopState.isEnabled
                ? NSLocalizedString("On", comment: "")
                : NSLocalizedString("Off", comment: "")

            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron
        case .language:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString("Language", comment: "")

            cell.detailTitleLabel.text = viewModel.currentLanguage

            cell.setAccessibilityIdentifier(item.accessibilityIdentifier)
            cell.disclosureType = .chevron
        }
    }

    @objc private func preferredContentSizeChanged(_ notification: Notification) {
        if let newContentSizeCategory = notification.userInfo?[UIContentSizeCategory.newValueUserInfoKey]
            as? UIContentSizeCategory
        {
            contentSizeCategory = newContentSizeCategory
        }
    }
}

private extension UIContentSizeCategory {
    var isLarge: Bool {
        (self > .extraExtraExtraLarge) || (self > .accessibilityLarge)
    }
}
