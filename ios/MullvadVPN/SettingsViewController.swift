//
//  SettingsViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import SafariServices
import UIKit

protocol SettingsViewControllerDelegate: AnyObject {
    func settingsViewControllerDidFinish(_ controller: SettingsViewController)
}

class SettingsViewController: UITableViewController, SettingsDataSourceDelegate,
    SFSafariViewControllerDelegate
{
    weak var delegate: SettingsViewControllerDelegate?
    private var dataSource: SettingsDataSource?
    private let interactor: SettingsInteractor

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    init(interactor: SettingsInteractor) {
        self.interactor = interactor
        super.init(style: .grouped)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "Settings",
            value: "Settings",
            comment: ""
        )
        navigationItem.rightBarButtonItem = UIBarButtonItem(
            barButtonSystemItem: .done,
            target: self,
            action: #selector(handleDismiss)
        )

        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 60

        dataSource = SettingsDataSource(
            tableView: tableView,
            interactor: interactor
        ) { [weak self] tableView, indexPath, itemIdentifier in
            return self?.getCell(for: itemIdentifier, indexPath: indexPath)
        }
        dataSource?.delegate = self
    }

    private func getCell(
        for item: SettingsDataSource.Item,
        indexPath: IndexPath
    ) -> UITableViewCell {
        switch item {
        case .account:
            let cell = tableView.dequeueReusableCell(
                withIdentifier: SettingsDataSource.CellReuseIdentifiers.accountCell.rawValue,
                for: indexPath
            ) as! SettingsAccountCell
            cell.titleLabel.text = NSLocalizedString(
                "ACCOUNT_CELL_LABEL",
                tableName: "Settings",
                value: "Account",
                comment: ""
            )
            cell.accountExpiryDate = interactor.deviceState.accountData?.expiry
            cell.accessibilityIdentifier = "AccountCell"
            cell.disclosureType = .chevron

            return cell

        case .preferences:
            let cell = tableView.dequeueReusableCell(
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
            let cell = tableView.dequeueReusableCell(
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
            let cell = tableView.dequeueReusableCell(
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
            let cell = tableView.dequeueReusableCell(
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

    // MARK: - IBActions

    @IBAction func handleDismiss() {
        delegate?.settingsViewControllerDidFinish(self)
    }

    // MARK: - SettingsDataSourceDelegate

    func settingsDataSource(
        _ dataSource: SettingsDataSource,
        didSelectItem item: SettingsDataSource.Item
    ) {
        if let route = item.navigationRoute {
            let settingsNavigationController = navigationController as? SettingsNavigationController

            settingsNavigationController?.navigate(to: route, animated: true)
        } else if case .faq = item {
            let safariViewController = SFSafariViewController(
                url: ApplicationConfiguration
                    .faqAndGuidesURL
            )
            safariViewController.delegate = self

            present(safariViewController, animated: true)
        }
    }

    // MARK: - SFSafariViewControllerDelegate

    func safariViewControllerDidFinish(_ controller: SFSafariViewController) {
        controller.dismiss(animated: true)
    }

    func safariViewControllerWillOpenInBrowser(_ controller: SFSafariViewController) {
        controller.dismiss(animated: false)
    }
}

extension SettingsDataSource.Item {
    var navigationRoute: SettingsNavigationRoute? {
        switch self {
        case .account:
            return .account
        case .preferences:
            return .preferences
        case .version:
            return nil
        case .problemReport:
            return .problemReport
        case .faq:
            return nil
        }
    }
}
