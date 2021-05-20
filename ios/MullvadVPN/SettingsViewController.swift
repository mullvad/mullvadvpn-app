//
//  SettingsViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

protocol SettingsViewControllerDelegate: AnyObject {
    func settingsViewControllerDidFinish(_ controller: SettingsViewController)
}

class SettingsViewController: UITableViewController, AccountObserver {

    weak var delegate: SettingsViewControllerDelegate?

    private enum CellIdentifier: String {
        case accountCell = "AccountCell"
        case basicCell = "BasicCell"
    }

    private let staticDataSource = SettingsTableViewDataSource()

    private weak var accountRow: StaticTableViewRow?
    private var accountExpiryObserver: NSObjectProtocol?

    private var settingsNavigationController: SettingsNavigationController? {
        return self.navigationController as? SettingsNavigationController
    }

    init() {
        super.init(style: .grouped)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 60
        tableView.sectionHeaderHeight = 18
        tableView.sectionFooterHeight = 18

        tableView.dataSource = staticDataSource
        tableView.delegate = staticDataSource

        tableView.register(SettingsAccountCell.self, forCellReuseIdentifier: CellIdentifier.accountCell.rawValue)
        tableView.register(SettingsCell.self, forCellReuseIdentifier: CellIdentifier.basicCell.rawValue)

        navigationItem.title = NSLocalizedString("Settings", comment: "Navigation title")
        navigationItem.largeTitleDisplayMode = .always
        navigationItem.rightBarButtonItem = UIBarButtonItem(barButtonSystemItem: .done, target: self, action: #selector(handleDismiss))

        Account.shared.addObserver(self)
        setupDataSource()
    }

    // MARK: - AccountObserver

    func account(_ account: Account, didUpdateExpiry expiry: Date) {
        guard let accountRow = accountRow else { return }

        staticDataSource.reloadRows([accountRow], with: .none)
    }

    func account(_ account: Account, didLoginWithToken token: String, expiry: Date) {
        // no-op
    }

    func accountDidLogout(_ account: Account) {
        // no-op
    }

    // MARK: - IBActions

    @IBAction func handleDismiss() {
        delegate?.settingsViewControllerDidFinish(self)
    }

    // MARK: - Private

    private func setupDataSource() {
        if Account.shared.isLoggedIn {
            let topSection = StaticTableViewSection()
            let accountRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.accountCell.rawValue) { (_, cell) in
                let cell = cell as! SettingsAccountCell

                cell.titleLabel.text = NSLocalizedString("Account", comment: "")
                cell.accountExpiryDate = Account.shared.expiry
                cell.accessibilityIdentifier = "AccountCell"
                cell.accessoryType = .disclosureIndicator
            }

            accountRow.actionBlock = { [weak self] (indexPath) in
                self?.settingsNavigationController?.navigate(to: .account, animated: true)
            }

            let preferencesRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.basicCell.rawValue) { (_, cell) in
                let cell = cell as! SettingsCell
                cell.titleLabel.text = NSLocalizedString("Preferences", comment: "")
                cell.accessoryType = .disclosureIndicator
            }

            preferencesRow.actionBlock = { [weak self] (indexPath) in
                self?.settingsNavigationController?.navigate(to: .preferences, animated: true)
            }

            let wireguardKeyRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.basicCell.rawValue) { (_, cell) in
                let cell = cell as! SettingsCell

                cell.titleLabel.text = NSLocalizedString("WireGuard key", comment: "")
                cell.accessibilityIdentifier = "WireGuardKeyCell"
                cell.accessoryType = .disclosureIndicator
            }

            wireguardKeyRow.actionBlock = { [weak self] (indexPath) in
                self?.settingsNavigationController?.navigate(to: .wireguardKeys, animated: true)
            }

            self.accountRow = accountRow

            topSection.addRows([accountRow, preferencesRow, wireguardKeyRow])
            staticDataSource.addSections([topSection])
        }

        let middleSection = StaticTableViewSection()
        let versionRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.basicCell.rawValue) { (_, cell) in
            let cell = cell as! SettingsCell
            cell.titleLabel.text = NSLocalizedString("App version", comment: "")
            cell.detailTitleLabel.text = Bundle.main.productVersion
        }
        versionRow.isSelectable = false

        middleSection.addRows([versionRow])
        staticDataSource.addSections([middleSection])

        #if DEBUG
        let logStreamerRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.basicCell.rawValue) { (_, cell) in
            let cell = cell as! SettingsCell

            cell.titleLabel.text = NSLocalizedString("App logs", comment: "")
        }
        logStreamerRow.actionBlock = { [weak self] (indexPath) in
            let logController = LogStreamerViewController(fileURLs: ApplicationConfiguration.logFileURLs)
            let navController = UINavigationController(rootViewController: logController)

            navController.modalPresentationStyle = .fullScreen

            self?.present(navController, animated: true)
        }
        middleSection.addRows([logStreamerRow])
        #endif

        let bottomSection = StaticTableViewSection()

        let problemReportRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.basicCell.rawValue) { (indexPath, cell) in
            let cell = cell as! SettingsCell

            cell.titleLabel.text = NSLocalizedString("Report a problem", comment: "")
            cell.accessoryType = .disclosureIndicator
        }

        problemReportRow.actionBlock = { [weak self] (indexPath) in
            self?.settingsNavigationController?.navigate(to: .problemReport, animated: true)
        }

        bottomSection.addRows([problemReportRow])
        staticDataSource.addSections([bottomSection])
    }

}

class SettingsTableViewDataSource: StaticTableViewDataSource {

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, heightForHeaderInSection section: Int) -> CGFloat {
        return 24
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        return 0.01
    }

}
