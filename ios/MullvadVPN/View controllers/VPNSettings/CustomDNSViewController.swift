//
//  CustomDNSViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-11-09.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

class CustomDNSViewController: UITableViewController {
    private let interactor: VPNSettingsInteractor
    private var dataSource: CustomDNSDataSource?
    private let alertPresenter: AlertPresenter

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    init(interactor: VPNSettingsInteractor, alertPresenter: AlertPresenter) {
        self.interactor = interactor
        self.alertPresenter = alertPresenter

        super.init(style: .grouped)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        tableView.setAccessibilityIdentifier(.dnsSettingsTableView)
        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 60
        tableView.estimatedSectionHeaderHeight = tableView.estimatedRowHeight

        dataSource = CustomDNSDataSource(tableView: tableView)
        dataSource?.delegate = self

        navigationItem.title = NSLocalizedString("DNS settings", comment: "")
        if navigationItem.rightBarButtonItem != nil {
            navigationItem.leftBarButtonItem = editButtonItem
            navigationItem.leftBarButtonItem?.setAccessibilityIdentifier(.dnsSettingsEditButton)
        } else {
            navigationItem.rightBarButtonItem = editButtonItem
            navigationItem.rightBarButtonItem?.setAccessibilityIdentifier(.dnsSettingsEditButton)
        }

        interactor.tunnelSettingsDidChange = { [weak self] newSettings in
            self?.dataSource?.update(from: newSettings)
        }
        dataSource?.update(from: interactor.tunnelSettings)

        tableView.tableHeaderView = UIView(
            frame: CGRect(
                origin: .zero,
                size: CGSize(width: 0, height: UIMetrics.TableView.emptyHeaderHeight)
            ))
    }

    override func setEditing(_ editing: Bool, animated: Bool) {
        super.setEditing(editing, animated: animated)

        dataSource?.setEditing(editing, animated: animated)

        navigationItem.setHidesBackButton(editing, animated: animated)
        if navigationItem.rightBarButtonItem != editButtonItem {
            navigationItem.rightBarButtonItem?.isHidden = editing
        }

        // Disable swipe to dismiss when editing
        isModalInPresentation = editing
    }

    private func showInfo(with message: NSAttributedString) {
        let presentation = AlertPresentation(
            id: "vpn-settings-content-blockers-alert",
            icon: .info,
            attributedMessage: message,
            buttons: [
                AlertAction(
                    title: NSLocalizedString("Got it!", comment: ""),
                    style: .default
                )
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }
}

extension CustomDNSViewController: @preconcurrency DNSSettingsDataSourceDelegate {
    func didChangeViewModel(_ viewModel: VPNSettingsViewModel) {
        interactor.updateSettings([.dnsSettings(viewModel.asDNSSettings())])
    }

    func showInfo(for item: VPNSettingsInfoButtonItem) {
        showInfo(
            with: NSAttributedString(
                markdownString: item.description,
                options: MarkdownStylingOptions(font: .preferredFont(forTextStyle: .body))
            ))
    }
}
