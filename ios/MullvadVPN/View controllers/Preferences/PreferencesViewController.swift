//
//  PreferencesViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 19/05/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import RelayCache
import UIKit

class PreferencesViewController: UITableViewController, PreferencesDataSourceDelegate {
    private let interactor: PreferencesInteractor
    private var dataSource: PreferencesDataSource?
    private let alertPresenter = AlertPresenter()
    private var cachedRelays: CachedRelays?

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
        tableView.estimatedSectionHeaderHeight = tableView.estimatedRowHeight
        tableView.allowsSelectionDuringEditing = true

        dataSource = PreferencesDataSource(tableView: tableView)
        dataSource?.delegate = self

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "Preferences",
            value: "VPN settings",
            comment: ""
        )
        navigationItem.rightBarButtonItem = editButtonItem

        interactor.tunnelSettingsDidChange = { [weak self] newSettings in
            self?.dataSource?.update(from: newSettings)
        }
        dataSource?.update(from: interactor.tunnelSettings)
        dataSource?.setAvailablePortRanges(cachedRelays?.relays.wireguard.portRanges ?? [])

        tableView.tableHeaderView =
            UIView(frame: .init(origin: .zero, size: .init(width: 0, height: UIMetrics.sectionSpacing)))
    }

    override func setEditing(_ editing: Bool, animated: Bool) {
        _ = dataSource?.revertWireGuardPortCellToLastSelection()

        super.setEditing(editing, animated: animated)

        dataSource?.setEditing(editing, animated: animated)

        navigationItem.setHidesBackButton(editing, animated: animated)

        // Disable swipe to dismiss when editing
        isModalInPresentation = editing
    }

    func setCachedRelays(_ cachedRelays: CachedRelays) {
        self.cachedRelays = cachedRelays
        dataSource?.setAvailablePortRanges(cachedRelays.relays.wireguard.portRanges)
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

    private func wireGuardPortRangesToString(_ ranges: [[UInt16]]) -> String {
        return ranges
            .compactMap { range in
                if let minPort = range.first, let maxPort = range.last {
                    return minPort == maxPort ? String(minPort) : "\(minPort)-\(maxPort)"
                } else {
                    return nil
                }
            }
            .joined(separator: ", ")
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
        didPressInfoButton item: PreferencesDataSource.InfoButtonItem?
    ) {
        var message = ""

        switch item {
        case .contentBlockers:
            message = NSLocalizedString(
                "PREFERENCES_CONTENT_BLOCKERS_GENERAL",
                tableName: "ContentBlockers",
                value: "When this feature is enabled it stops the device from contacting certain domains or websites known for distributing ads, malware, trackers and more. This might cause issues on certain websites, services, and programs.",
                comment: ""
            )

        case .blockMalware:
            message = NSLocalizedString(
                "PREFERENCES_CONTENT_BLOCKERS_MALWARE",
                tableName: "ContentBlockers",
                value: "Warning: The malware blocker is not an anti-virus and should not be treated as such, this is just an extra layer of protection.",
                comment: ""
            )

        case .wireGuardPorts:
            let portsString = wireGuardPortRangesToString(cachedRelays?.relays.wireguard.portRanges ?? [])

            message = NSLocalizedString(
                "PREFERENCES_WIRE_GUARD_PORTS_GENERAL",
                tableName: "WireGuardPorts",
                value: "The automatic setting will randomly choose from the valid port ranges shown below.\n\nThe custom port can be any value inside the valid ranges:\n\n\(portsString)",
                comment: ""
            )

        default:
            assertionFailure("No matching InfoButtonItem")
        }

        showContentBlockerInfo(with: message)
    }

    func preferencesDataSource(_ dataSource: PreferencesDataSource, didSelectPort port: UInt16?) {
        interactor.setPort(port)
    }
}
