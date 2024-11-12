//
//  SettingsViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
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
    private let alertPresenter: AlertPresenter

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    init(interactor: SettingsInteractor, alertPresenter: AlertPresenter) {
        self.interactor = interactor
        self.alertPresenter = alertPresenter

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

        let doneButton = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction(handler: { [weak self] _ in
                guard let self else { return }

                delegate?.settingsViewControllerDidFinish(self)
            })
        )
        doneButton.accessibilityIdentifier = .settingsDoneButton
        navigationItem.rightBarButtonItem = doneButton

        tableView.accessibilityIdentifier = .settingsTableView
        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 60

        dataSource = SettingsDataSource(tableView: tableView, interactor: interactor)
        dataSource?.delegate = self
    }
}

extension SettingsViewController: SettingsDataSourceDelegate {
    func didSelectItem(item: SettingsDataSource.Item) {
        guard let route = item.navigationRoute else { return }
        delegate?.settingsViewController(self, didRequestRoutePresentation: route)
    }

    func showInfo(for item: SettingsInfoButtonItem) {
        let presentation = AlertPresentation(
            id: "settings-info-alert",
            icon: .info,
            message: item.description,
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "SETTINGS_INFO_ALERT_OK_ACTION",
                        tableName: "Settings",
                        value: "Got it!",
                        comment: ""
                    ),
                    style: .default
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }

    func showPrompt(
        for item: DAITASettingsPromptItem,
        onSave: @escaping () -> Void,
        onDiscard: @escaping () -> Void
    ) {
        let presentation = AlertPresentation(
            id: "settings-daita-prompt",
            accessibilityIdentifier: .daitaPromptAlert,
            icon: .info,
            message: NSLocalizedString(
                "SETTINGS_DAITA_ENABLE_TEXT",
                tableName: "DAITA",
                value: item.description,
                comment: ""
            ),
            buttons: [
                AlertAction(
                    title: String(format: NSLocalizedString(
                        "SETTINGS_DAITA_ENABLE_OK_ACTION",
                        tableName: "DAITA",
                        value: "Enable %@",
                        comment: ""
                    ), item.title),
                    style: .default,
                    accessibilityId: .daitaConfirmAlertEnableButton,
                    handler: { onSave() }
                ),
                AlertAction(
                    title: NSLocalizedString(
                        "SETTINGS_DAITA_ENABLE_CANCEL_ACTION",
                        tableName: "DAITA",
                        value: "Back",
                        comment: ""
                    ),
                    style: .default,
                    handler: { onDiscard() }
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }
}

extension SettingsDataSource.Item {
    var navigationRoute: SettingsNavigationRoute? {
        switch self {
        case .vpnSettings:
            return .vpnSettings
        case .version, .daita, .daitaDirectOnly:
            return nil
        case .problemReport:
            return .problemReport
        case .faq:
            return .faq
        case .apiAccess:
            return .apiAccess
        case .multihop:
            return .multihop
        }
    }
}
