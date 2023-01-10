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
    func settingsViewController(
        _ controller: SettingsViewController,
        didRequestRoutePresentation route: SettingsNavigationRoute
    )
}

class SettingsViewController: UITableViewController, SettingsDataSourceDelegate {
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
        guard let route = item.navigationRoute else { return }

        delegate?.settingsViewController(self, didRequestRoutePresentation: route)
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
            return .faq
        }
    }
}
