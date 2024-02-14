//
//  CustomDNSViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-11-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

class CustomDNSViewController: UITableViewController, PreferencesDataSourceDelegate {
    private let interactor: PreferencesInteractor
    private var dataSource: CustomDNSDataSource?
    private let alertPresenter: AlertPresenter

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    init(interactor: PreferencesInteractor, alertPresenter: AlertPresenter) {
        self.interactor = interactor
        self.alertPresenter = alertPresenter

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
        tableView.estimatedSectionHeaderHeight = tableView.estimatedRowHeight

        dataSource = CustomDNSDataSource(tableView: tableView)
        dataSource?.delegate = self

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "Preferences",
            value: "DNS settings",
            comment: ""
        )
        navigationItem.rightBarButtonItem = editButtonItem

        interactor.tunnelSettingsDidChange = { [weak self] newSettings in
            self?.dataSource?.update(from: newSettings)
        }
        dataSource?.update(from: interactor.tunnelSettings)

        tableView.tableHeaderView = UIView(frame: CGRect(
            origin: .zero,
            size: CGSize(width: 0, height: UIMetrics.TableView.sectionSpacing)
        ))
    }

    override func setEditing(_ editing: Bool, animated: Bool) {
        super.setEditing(editing, animated: animated)

        dataSource?.setEditing(editing, animated: animated)

        navigationItem.setHidesBackButton(editing, animated: animated)

        // Disable swipe to dismiss when editing
        isModalInPresentation = editing
    }

    private func showInfo(with message: String) {
        let presentation = AlertPresentation(
            id: "preferences-content-blockers-alert",
            icon: .info,
            message: message,
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "PREFERENCES_DNS_SETTINGS_OK_ACTION",
                        tableName: "ContentBlockers",
                        value: "Got it!",
                        comment: ""
                    ),
                    style: .default
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }

    // MARK: - PreferencesDataSourceDelegate

    func didChangeViewModel(_ viewModel: PreferencesViewModel) {
        interactor.updateSettings([.dnsSettings(viewModel.asDNSSettings())])
    }

    func showInfo(for item: PreferencesInfoButtonItem) {
        var message = ""

        switch item {
        case .contentBlockers:
            message = NSLocalizedString(
                "PREFERENCES_CONTENT_BLOCKERS_GENERAL",
                tableName: "ContentBlockers",
                value: """
                When this feature is enabled it stops the device from contacting certain \
                domains or websites known for distributing ads, malware, trackers and more. \
                This might cause issues on certain websites, services, and programs.
                """,
                comment: ""
            )

        case .blockMalware:
            message = NSLocalizedString(
                "PREFERENCES_CONTENT_BLOCKERS_MALWARE",
                tableName: "ContentBlockers",
                value: """
                Warning: The malware blocker is not an anti-virus and should not \
                be treated as such, this is just an extra layer of protection.
                """,
                comment: ""
            )

        default:
            assertionFailure("No matching InfoButtonItem")
        }

        showInfo(with: message)
    }

    func showDNSSettings() {
        // No op.
    }

    func didSelectWireGuardPort(_ port: UInt16?) {
        // No op.
    }
}
