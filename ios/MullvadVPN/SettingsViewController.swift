//
//  SettingsViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

enum SettingsNavigationRoute {
    case account
    case wireguardKeys
}

enum SettingsDismissReason {
    case none
    case userLoggedOut
}

protocol SettingsViewControllerDelegate: class {
    func settingsViewController(_ controller: SettingsViewController, didFinishWithReason reason: SettingsDismissReason)
}

class SettingsViewController: UITableViewController, AccountViewControllerDelegate {

    @IBOutlet var staticDataSource: SettingsTableViewDataSource!

    private enum CellIdentifier: String {
        case account = "Account"
        case appVersion = "AppVersion"
        case basicDisclosure = "BasicDisclosure"
        case basic = "Basic"
    }

    private weak var accountRow: StaticTableViewRow?
    private var accountExpiryObserver: NSObjectProtocol?

    weak var settingsDelegate: SettingsViewControllerDelegate?

    override func viewDidLoad() {
        super.viewDidLoad()

        accountExpiryObserver = NotificationCenter.default.addObserver(
            forName: Account.didUpdateAccountExpiryNotification,
            object: Account.shared,
            queue: OperationQueue.main) { [weak self] (note) in
                guard let accountRow = self?.accountRow else { return }

                self?.staticDataSource.reloadRows([accountRow], with: .none)
        }

        setupDataSource()
    }

    // MARK: - IBActions

    @IBAction func handleDismiss() {
        settingsDelegate?.settingsViewController(self, didFinishWithReason: .none)
    }

    // MARK: - Navigation

    func navigate(to route: SettingsNavigationRoute) {
        switch route {
        case .account:
            let controller = AccountViewController()
            controller.delegate = self

            navigationController?.pushViewController(controller, animated: true)

        case .wireguardKeys:
            let controller = WireguardKeysViewController()

            navigationController?.pushViewController(controller, animated: true)
        }
    }

    // MARK: - AccountViewControllerDelegate

    func accountViewControllerDidLogout(_ controller: AccountViewController) {
        settingsDelegate?.settingsViewController(self, didFinishWithReason: .userLoggedOut)
    }

    // MARK: - Private

    private func setupDataSource() {
        if Account.shared.isLoggedIn {
            let topSection = StaticTableViewSection()
            let accountRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.account.rawValue) { (_, cell) in
                let cell = cell as! SettingsAccountCell

                cell.accountExpiryDate = Account.shared.expiry
            }

            accountRow.actionBlock = { [weak self] (indexPath) in
                self?.navigate(to: .account)
            }

            let wireguardKeyRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.basicDisclosure.rawValue) { (_, cell) in
                let cell = cell as! SettingsBasicCell

                cell.titleLabel.text = NSLocalizedString("WireGuard key", comment: "")
                cell.accessibilityIdentifier = "WireGuardKeyCell"
            }

            wireguardKeyRow.actionBlock = { [weak self] (indexPath) in
                self?.navigate(to: .wireguardKeys)
            }

            self.accountRow = accountRow

            topSection.addRows([accountRow, wireguardKeyRow])
            staticDataSource.addSections([topSection])
        }

        let middleSection = StaticTableViewSection()
        let versionRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.appVersion.rawValue) { (_, cell) in
            let cell = cell as! SettingsAppVersionCell
            let version = Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String

            cell.versionLabel.text = version
        }
        versionRow.isSelectable = false

        middleSection.addRows([versionRow])
        staticDataSource.addSections([middleSection])

        #if DEBUG
        let logStreamerRow = StaticTableViewRow(reuseIdentifier: CellIdentifier.basic.rawValue) { (_, cell) in
            let cell = cell as! SettingsBasicCell

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
