//
//  SettingsViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import Routing
import UIKit

protocol SettingsViewControllerDelegate: AnyObject {
    func settingsViewControllerDidFinish(_ controller: SettingsViewController)
    func settingsViewController(
        _ controller: SettingsViewController,
        didRequestRoutePresentation route: SettingsNavigationRoute
    )
}

class SettingsViewController: UITableViewController {
    weak var delegate: SettingsViewControllerDelegate?
    private var dataSource: SettingsDataSource?
    private let interactor: SettingsInteractor

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
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

        navigationItem.title = NSLocalizedString("Settings", comment: "")

        let doneButton = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction(handler: { [weak self] _ in
                guard let self else { return }

                delegate?.settingsViewControllerDidFinish(self)
            })
        )
        doneButton.setAccessibilityIdentifier(.settingsDoneButton)
        navigationItem.rightBarButtonItem = doneButton

        tableView.setAccessibilityIdentifier(.settingsTableView)
        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 60

        interactor.didUpdateSettings = { [weak self] in
            self?.dataSource?.reload()
        }

        dataSource = SettingsDataSource(tableView: tableView, interactor: interactor)
        dataSource?.delegate = self
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        dataSource?.reload()
    }
}

extension SettingsViewController: @preconcurrency SettingsDataSourceDelegate {
    func didSelectItem(item: SettingsDataSource.Item) {
        guard let route = item.navigationRoute else { return }
        delegate?.settingsViewController(self, didRequestRoutePresentation: route)
    }
}

extension SettingsDataSource.Item {
    var navigationRoute: SettingsNavigationRoute? {
        switch self {
        case .vpnSettings:
            .vpnSettings
        case .changelog:
            .changelog
        case .problemReport:
            .problemReport
        case .faq:
            .faq
        case .apiAccess:
            .apiAccess
        case .daita:
            .daita
        case .multihop:
            .multihop
        case .language:
            .language
        }
    }
}
