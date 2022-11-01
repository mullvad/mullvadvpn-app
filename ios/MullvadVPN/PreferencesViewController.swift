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
    private let dataSource = PreferencesDataSource()

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

        dataSource.tableView = tableView
        dataSource.delegate = self

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "Preferences",
            value: "Preferences",
            comment: ""
        )
        navigationItem.rightBarButtonItem = editButtonItem

        interactor.dnsSettingsDidChange = { [weak self] newDNSSettings in
            self?.dataSource.update(from: newDNSSettings)
        }

        dataSource.update(from: interactor.dnsSettings)
    }

    override func setEditing(_ editing: Bool, animated: Bool) {
        dataSource.setEditing(editing, animated: animated)

        navigationItem.setHidesBackButton(editing, animated: animated)

        // Disable swipe to dismiss when editing
        isModalInPresentation = editing

        super.setEditing(editing, animated: animated)
    }

    // MARK: - PreferencesDataSourceDelegate

    func preferencesDataSource(
        _ dataSource: PreferencesDataSource,
        didChangeViewModel dataModel: PreferencesViewModel
    ) {
        let dnsSettings = dataModel.asDNSSettings()

        interactor.setDNSSettings(dnsSettings)
    }
}
