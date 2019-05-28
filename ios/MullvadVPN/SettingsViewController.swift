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
    }

    private var accountExpiryObserver: AccountExpiryRefresh.Observer?
    private weak var accountRow: StaticTableViewRow?

    override func viewDidLoad() {
        super.viewDidLoad()

        setupDataSource()
        startAccountExpiryUpdates()
    }

    // MARK: - IBActions

    @IBAction func handleDismiss() {
        dismiss(animated: true)
    }

    // MARK: - Private

    private func startAccountExpiryUpdates() {
        accountExpiryObserver = AccountExpiryRefresh.shared
            .startMonitoringUpdates(with: { [weak self] (expiryDate) in
                guard let self = self, let accountRow = self.accountRow else { return }

                self.staticDataSource.reloadRows([accountRow], with: .automatic)
            })
    }

    private func setupDataSource() {
        if Account.isLoggedIn {
            let topSection = StaticTableViewSection()
            let accountRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.account.rawValue) { (_, cell) in
                let cell = cell as! SettingsAccountCell

                cell.accountExpiryDate = Account.expiry
            }

            self.accountRow = accountRow

            topSection.addRows([accountRow])
            staticDataSource.addSections([topSection])
        }

        let middleSection = StaticTableViewSection()
        let versionRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.appVersion.rawValue) { (_, cell) in
            let cell = cell as! SettingsAppVersionCell
            let versionString = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String

            cell.versionLabel.text = versionString
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
