//
//  PreferencesViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 19/05/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class PreferencesViewController: UITableViewController, PreferencesDataSourceDelegate {
    private let interactor: PreferencesInteractor
    private var dataSource: PreferencesDataSource?
    private let alertPresenter = AlertPresenter()

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    init(interactor: PreferencesInteractor) {
        self.interactor = interactor
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

        dataSource = PreferencesDataSource(tableView: tableView)
        dataSource?.delegate = self

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "Preferences",
            value: "VPN settings",
            comment: ""
        )
        navigationItem.rightBarButtonItem = editButtonItem

        interactor.dnsSettingsDidChange = { [weak self] newDNSSettings in
            self?.dataSource?.update(from: newDNSSettings)
        }

        dataSource?.update(from: interactor.dnsSettings)
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        tableView.tableHeaderView =
            UIView(frame: .init(origin: .zero, size: .init(width: 0, height: UIMetrics.sectionSpacing)))
    }

    override func setEditing(_ editing: Bool, animated: Bool) {
        dataSource?.setEditing(editing, animated: animated)

        navigationItem.setHidesBackButton(editing, animated: animated)

        // Disable swipe to dismiss when editing
        isModalInPresentation = editing

        super.setEditing(editing, animated: animated)
    }

    private func showContentBlockerInfo(with message: String) {
        let alertController = UIAlertController(
            title: nil,
            message: message,
            preferredStyle: .alert
        )
        alertController.addAction(
            UIAlertAction(title: NSLocalizedString(
                "PREFERENCES_CONTENT_BLOCKERS_OK_ACTION",
                tableName: "ContentBlockers",
                value: "Got it!",
                comment: ""
            ), style: .cancel)
        )
        alertPresenter.enqueue(alertController, presentingController: self)
    }

    // MARK: - PreferencesDataSourceDelegate

    func preferencesDataSource(
        _ dataSource: PreferencesDataSource,
        didChangeViewModel dataModel: PreferencesViewModel
    ) {
        let dnsSettings = dataModel.asDNSSettings()

        interactor.setDNSSettings(dnsSettings)
    }

    func preferencesDataSource(
        _ dataSource: PreferencesDataSource,
        didPressInfoButton item: PreferencesDataSource.Item?
    ) {
        let message: String

        switch item {
        case .blockMalware:
            message = NSLocalizedString(
                "PREFERENCES_CONTENT_BLOCKERS_MALWARE",
                tableName: "ContentBlockers",
                value: "Warning: The malware blocker is not an anti-virus and should not be treated as such, this is just an extra layer of protection.",
                comment: ""
            )

        default:
            message = NSLocalizedString(
                "PREFERENCES_CONTENT_BLOCKERS_GENERAL",
                tableName: "ContentBlockers",
                value: "When this feature is enabled it stops the device from contacting certain domains or websites known for distributing ads, malware, trackers and more. This might cause issues on certain websites, services, and programs.",
                comment: ""
            )
        }

        showContentBlockerInfo(with: message)
    }
}
