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

    private func showInfo(with message: String) {
        let presentation = AlertPresentation(
            id: "vpn-settings-content-blockers-alert",
            icon: .info,
            message: message,
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

    private func humanReadablePortRepresentation(_ ranges: [[UInt16]]) -> String {
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
}

extension VPNSettingsViewController: VPNSettingsDataSourceDelegate {
    func didChangeViewModel(_ viewModel: VPNSettingsViewModel) {
        interactor.updateSettings(
            [
                .obfuscation(WireGuardObfuscationSettings(
                    state: viewModel.obfuscationState,
                    port: viewModel.obfuscationPort
                )),
                .quantumResistance(viewModel.quantumResistance),
                .multihop(viewModel.multihopState),
            ]
        )
    }

    // swiftlint:disable:next function_body_length
    func showInfo(for item: VPNSettingsInfoButtonItem) {
        var message = ""

        switch item {
        case .wireGuardPorts:
            let portsString = humanReadablePortRepresentation(
                interactor.cachedRelays?.relays.wireguard.portRanges ?? []
            )

            message = String(
                format: NSLocalizedString(
                    "VPN_SETTINGS_WIRE_GUARD_PORTS_GENERAL",
                    tableName: "WireGuardPorts",
                    value: """
                    The automatic setting will randomly choose from the valid port ranges shown below.
                    The custom port can be any value inside the valid ranges:
                    %@
                    """,
                    comment: ""
                ),
                portsString
            )

        case .wireGuardObfuscation:
            message = NSLocalizedString(
                "VPN_SETTINGS_WIRE_GUARD_OBFUSCATION_GENERAL",
                tableName: "WireGuardObfuscation",
                value: """
                Obfuscation hides the WireGuard traffic inside another protocol. \
                It can be used to help circumvent censorship and other types of filtering, \
                where a plain WireGuard connect would be blocked.
                """,
                comment: ""
            )

        case .wireGuardObfuscationPort:
            message = NSLocalizedString(
                "VPN_SETTINGS_WIRE_GUARD_OBFUSCATION_PORT_GENERAL",
                tableName: "WireGuardObfuscation",
                value: "Which TCP port the UDP-over-TCP obfuscation protocol should connect to on the VPN server.",
                comment: ""
            )

        case .quantumResistance:
            message = NSLocalizedString(
                "VPN_SETTINGS_QUANTUM_RESISTANCE_GENERAL",
                tableName: "QuantumResistance",
                value: """
                This feature makes the WireGuard tunnel resistant to potential attacks from quantum computers.
                It does this by performing an extra key exchange using a quantum safe algorithm and mixing \
                the result into WireGuard’s regular encryption.
                This extra step uses approximately 500 kiB of traffic every time a new tunnel is established.
                """,
                comment: ""
            )
        default:
            assertionFailure("No matching InfoButtonItem")
        }

        showInfo(with: message)
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

    func showMultihopConfirmation(_ onSave: @escaping () -> Void, _ onDiscard: @escaping () -> Void) {
        let presentation = AlertPresentation(
            id: "multihop-confirm-alert",
            accessibilityIdentifier: .multihopPromptAlert,
            icon: .info,
            message: NSLocalizedString(
                "MULTIHOP_CONFIRM_ALERT_TEXT",
                tableName: "Multihop",
                value: "This setting increases latency. Use only if needed.",
                comment: ""
            ),
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "MULTIHOP_CONFIRM_ALERT_ENABLE_BUTTON",
                        tableName: "Multihop",
                        value: "Enable anyway",
                        comment: ""
                    ),
                    style: .destructive,
                    accessibilityId: .multihopConfirmAlertEnableButton,
                    handler: {
                        onSave()
                    }
                ),
                AlertAction(
                    title: NSLocalizedString(
                        "MULTIHOP_CONFIRM_ALERT_BACK_BUTTON",
                        tableName: "Multihop",
                        value: "Back",
                        comment: ""
                    ),
                    style: .default,
                    accessibilityId: .multihopConfirmAlertBackButton,
                    handler: {
                        onDiscard()
                    }
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }
}
