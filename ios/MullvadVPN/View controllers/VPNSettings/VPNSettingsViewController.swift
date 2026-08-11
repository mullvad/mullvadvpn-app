//
//  VPNSettingsViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 19/05/2021.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadSettings
import SwiftUI
import UIKit

protocol VPNSettingsViewControllerDelegate: AnyObject {
    func showIPOverrides()
    func showAntiCensorshipSettings()
}

class VPNSettingsViewController: UITableViewController {
    private let interactor: VPNSettingsInteractor
    private var dataSource: VPNSettingsDataSource?
    private let alertPresenter: AlertPresenter
    private let section: VPNSettingsSection?
    weak var delegate: VPNSettingsViewControllerDelegate?

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    init(
        interactor: VPNSettingsInteractor,
        alertPresenter: AlertPresenter,
        section: VPNSettingsSection?
    ) {
        self.interactor = interactor
        self.alertPresenter = alertPresenter
        self.section = section
        super.init(style: .grouped)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        tableView.setAccessibilityIdentifier(.vpnSettingsTableView)
        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 60
        tableView.estimatedSectionHeaderHeight = tableView.estimatedRowHeight
        tableView.allowsMultipleSelection = true

        dataSource = VPNSettingsDataSource(
            tableView: tableView,
            section: section
        )

        dataSource?.delegate = self

        navigationItem.title = NSLocalizedString("VPN settings", comment: "")

        interactor.tunnelSettingsDidChange = { [weak self] newSettings in
            self?.dataSource?.reload(from: newSettings)
        }
        dataSource?.update(from: interactor.tunnelSettings)

        dataSource?.setAvailablePortRanges(interactor.cachedRelays?.relays.wireguard.portRanges ?? [])
        interactor.cachedRelaysDidChange = { [weak self] cachedRelays in
            self?.dataSource?.setAvailablePortRanges(cachedRelays.relays.wireguard.portRanges)
        }

        let showsSingleSection = section != nil
        tableView.tableHeaderView = UIView(
            frame: CGRect(
                origin: .zero,
                size: CGSize(width: 0, height: showsSingleSection ? 0 : UIMetrics.TableView.emptyHeaderHeight)
            ))
    }
}

extension VPNSettingsViewController: @preconcurrency VPNSettingsDataSourceDelegate {
    func obfuscationSettingsAreValid(
        _ settings: MullvadSettings.WireGuardObfuscationSettings,
        completion: @escaping (Bool) -> Void
    ) {
        var tunnelSettings = interactor.tunnelManager.settings
        tunnelSettings.wireGuardObfuscation = settings

        do {
            _ = try interactor.tunnelManager.selectRelays(tunnelSettings: tunnelSettings)
            completion(true)
        } catch let error as NoRelaysSatisfyingConstraintsError where error.reason == .noObfuscatedRelaysFound {
            showObfuscationSettingsIncompatibilityWarning(for: settings.state, completion: completion)
        } catch {
            completion(true)
        }
    }

    func humanReadablePortRepresentation() -> String {
        let ranges = interactor.cachedRelays?.relays.wireguard.portRanges ?? []
        return
            ranges
            .compactMap { range in
                if let minPort = range.first, let maxPort = range.last {
                    return minPort == maxPort ? String(minPort) : "\(minPort)-\(maxPort)"
                } else {
                    return nil
                }
            }
            .joined(separator: ", ")
    }

    func showObfuscationSettingsIncompatibilityWarning(
        for state: WireGuardObfuscationState,
        completion: @escaping ((Bool) -> Void)
    ) {
        let presentation = AlertPresentation(
            id: "vpn-settings-obfuscation-alert",
            accessibilityIdentifier: .wireGuardObfuscationIncompatibilityAlert,
            icon: .warning,
            message: BlockedStateString.Message.obfuscation.description,
            buttons: [
                AlertAction(
                    title: BlockedStateString.Button.obfuscation(state).description,
                    style: .destructive,
                    accessibilityId: .obfuscationConfirmAlertEnableButton,
                    handler: { completion(true) }
                ),
                AlertAction(
                    title: NSLocalizedString("Cancel", comment: ""),
                    style: .default,
                    handler: { completion(false) }
                ),
            ]
        )
        alertPresenter.showAlert(presentation: presentation, animated: true)
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
                    title: NSLocalizedString("Got it!", comment: ""),
                    style: .default
                )
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }

    func showDNSSettings() {
        let viewController = CustomDNSViewController(interactor: interactor, alertPresenter: alertPresenter)
        navigationController?.pushViewController(viewController, animated: true)
    }

    func showIPOverrides() {
        delegate?.showIPOverrides()
    }

    func showAntiCensorshipSettings() {
        delegate?.showAntiCensorshipSettings()
    }

    func didSelectWireGuardPort(_ port: UInt16?) {
        interactor.setPort(port)
    }
}
