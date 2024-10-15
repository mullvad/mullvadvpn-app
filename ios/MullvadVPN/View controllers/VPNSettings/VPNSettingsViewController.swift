//
//  VPNSettingsViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 19/05/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

protocol VPNSettingsViewControllerDelegate: AnyObject {
    func showIPOverrides()
}

class VPNSettingsViewController: UITableViewController {
    private let interactor: VPNSettingsInteractor
    private var dataSource: VPNSettingsDataSource?
    private let alertPresenter: AlertPresenter

    weak var delegate: VPNSettingsViewControllerDelegate?

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

        tableView.accessibilityIdentifier = .vpnSettingsTableView
        tableView.backgroundColor = .secondaryColor
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 60
        tableView.estimatedSectionHeaderHeight = tableView.estimatedRowHeight
        tableView.allowsMultipleSelection = true

        dataSource = VPNSettingsDataSource(tableView: tableView)
        dataSource?.delegate = self

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "VPNSettings",
            value: "VPN settings",
            comment: ""
        )

        interactor.tunnelSettingsDidChange = { [weak self] newSettings in
            self?.dataSource?.update(from: newSettings)
        }
        dataSource?.update(from: interactor.tunnelSettings)

        dataSource?.setAvailablePortRanges(interactor.cachedRelays?.relays.wireguard.portRanges ?? [])
        interactor.cachedRelaysDidChange = { [weak self] cachedRelays in
            self?.dataSource?.setAvailablePortRanges(cachedRelays.relays.wireguard.portRanges)
        }

        tableView.tableHeaderView = UIView(frame: CGRect(
            origin: .zero,
            size: CGSize(width: 0, height: UIMetrics.TableView.sectionSpacing)
        ))
    }
}

extension VPNSettingsViewController: VPNSettingsDataSourceDelegate {
    func humanReadablePortRepresentation() -> String {
        let ranges = interactor.cachedRelays?.relays.wireguard.portRanges ?? []
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

    func didUpdateTunnelSettings(_ update: TunnelSettingsUpdate) {
        interactor.updateSettings([update])
    }

    func showInfo(for item: VPNSettingsInfoButtonItem) {
        let presentation = AlertPresentation(
            id: "vpn-settings-content-blockers-alert",
            icon: .info,
            message: item.description,
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "VPN_SETTINGS_VPN_SETTINGS_OK_ACTION",
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

    func showDetails(for: VPNSettingsDetailsButtonItem) {
        // TODO: When ready, add navigation to detail views for selecting obfuscation options for
        // UDP-over-TCP and shadowsocks.
    }

    func showDNSSettings() {
        let viewController = CustomDNSViewController(interactor: interactor, alertPresenter: alertPresenter)
        navigationController?.pushViewController(viewController, animated: true)
    }

    func showIPOverrides() {
        delegate?.showIPOverrides()
    }

    func didSelectWireGuardPort(_ port: UInt16?) {
        interactor.setPort(port)
    }
}
