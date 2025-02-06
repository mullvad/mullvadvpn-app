//
//  VPNSettingsViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 19/05/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import SwiftUI
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

        tableView.setAccessibilityIdentifier(.vpnSettingsTableView)
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
            self?.dataSource?.reload(from: newSettings)
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

extension VPNSettingsViewController: @preconcurrency VPNSettingsDataSourceDelegate {
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

    func showDetails(for item: VPNSettingsDetailsButtonItem) {
        switch item {
        case .udpOverTcp:
            showUDPOverTCPObfuscationSettings()
        case .wireguardOverShadowsocks:
            showShadowsocksObfuscationSettings()
        }
    }

    func showDNSSettings() {
        let viewController = CustomDNSViewController(interactor: interactor, alertPresenter: alertPresenter)
        navigationController?.pushViewController(viewController, animated: true)
    }

    func showIPOverrides() {
        delegate?.showIPOverrides()
    }

    private func showUDPOverTCPObfuscationSettings() {
        let viewModel = TunnelUDPOverTCPObfuscationSettingsViewModel(tunnelManager: interactor.tunnelManager)
        let view = UDPOverTCPObfuscationSettingsView(viewModel: viewModel)
        let vc = UIHostingController(rootView: view)
        vc.title = NSLocalizedString(
            "UDP_OVER_TCP_TITLE",
            tableName: "VPNSettings",
            value: "UDP-over-TCP",
            comment: ""
        )
        navigationController?.pushViewController(vc, animated: true)
    }

    private func showShadowsocksObfuscationSettings() {
        let viewModel = TunnelShadowsocksObfuscationSettingsViewModel(tunnelManager: interactor.tunnelManager)
        let view = ShadowsocksObfuscationSettingsView(viewModel: viewModel)
        let vc = UIHostingController(rootView: view)
        vc.title = NSLocalizedString(
            "SHADOWSOCKS_TITLE",
            tableName: "VPNSettings",
            value: "Shadowsocks",
            comment: ""
        )
        navigationController?.pushViewController(vc, animated: true)
    }

    func didSelectWireGuardPort(_ port: UInt16?) {
        interactor.setPort(port)
    }

    func showLocalNetworkSharingWarning(_ enable: Bool, completion: @escaping (Bool) -> Void) {
        if interactor.tunnelManager.tunnelStatus.state.isSecured {
            let description = NSLocalizedString(
                "VPN_SETTINGS_LOCAL_NETWORK_SHARING_WARNING",
                tableName: "LocalNetworkSharing",
                value: """
                \(
                    enable
                        ? "Enabling"
                        : "Disabling"
                ) “Local network sharing” requires restarting the VPN connection, which will disconnect you and briefly expose your traffic. 
                To prevent this, manually enable Airplane Mode and turn off Wi-Fi before continuing. 

                Would you like to continue to enable “Local network sharing”?
                """,
                comment: ""
            )

            let presentation = AlertPresentation(
                id: "vpn-settings-local-network-sharing-warning",
                icon: .info,
                message: description,
                buttons: [
                    AlertAction(
                        title: NSLocalizedString(
                            "VPN_SETTINGS_LOCAL_NETWORK_SHARING_OK_ACTION",
                            tableName: "ContentBlockers",
                            value: "Yes, continue",
                            comment: ""
                        ),
                        style: .destructive,
                        handler: { completion(true) }
                    ),
                    AlertAction(
                        title: NSLocalizedString(
                            "VPN_SETTINGS_LOCAL_NETWORK_SHARING_CANCEL_ACTION",
                            tableName: "ContentBlockers",
                            value: "Cancel",
                            comment: ""
                        ),
                        style: .default,
                        handler: { completion(false) }
                    ),
                ]
            )

            alertPresenter.showAlert(presentation: presentation, animated: true)
        } else {
            completion(true)
        }
    }
}
