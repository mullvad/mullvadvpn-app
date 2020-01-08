//
//  SettingsViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit
import Foundation

class SettingsViewController: UITableViewController {

    @IBOutlet var staticDataSource: SettingsTableViewDataSource!

    private enum CellIdentifier: String {
        case account = "Account"
        case appVersion = "AppVersion"
        case basicDisclosure = "BasicDisclosure"
    }

    private weak var accountRow: StaticTableViewRow?

    override func viewDidLoad() {
        super.viewDidLoad()

        setupDataSource()
    }

    // MARK: - IBActions

    @IBAction func handleDismiss() {
        dismiss(animated: true)
    }

    // MARK: - Private

    private func setupDataSource() {
        if Account.shared.isLoggedIn {
            let topSection = StaticTableViewSection()
            let accountRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.account.rawValue) { (_, cell) in
                let cell = cell as! SettingsAccountCell

                cell.accountExpiryDate = Account.shared.expiry
            }

            let wireguardKeyRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.basicDisclosure.rawValue) { (_, cell) in
                let cell = cell as! SettingsBasicCell

                cell.titleLabel.text = NSLocalizedString("WireGuard key", comment: "")
            }

            wireguardKeyRow.actionBlock = { [weak self] (indexPath) in
                self?.performSegue(
                    withIdentifier: SegueIdentifier.Settings.showWireguardKeys.rawValue,
                    sender: nil)
            }

            self.accountRow = accountRow

            topSection.addRows([accountRow, wireguardKeyRow])
            staticDataSource.addSections([topSection])
        }

        let middleSection = StaticTableViewSection()
        let versionRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.appVersion.rawValue) { (_, cell) in
            let cell = cell as! SettingsAppVersionCell

            cell.versionLabel.text = Bundle.main.mullvadVersion
        }
        versionRow.isSelectable = false

        middleSection.addRows([versionRow])
        staticDataSource.addSections([middleSection])
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
