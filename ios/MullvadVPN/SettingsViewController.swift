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

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    private let dataSource: SettingsDataSource

    init(interactor: SettingsInteractor) {
        dataSource = SettingsDataSource(interactor: interactor)

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

        dataSource.tableView = tableView
        dataSource.delegate = self
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
