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
    func didUpdateTunnelSettings(_ update: TunnelSettingsUpdate) {
        interactor.updateSettings([update])
    }

    // swiftlint:disable:next function_body_length
    func showInfo(for item: VPNSettingsInfoButtonItem) { switch item {
    case .wireGuardPorts:
        let portsString = humanReadablePortRepresentation(
            interactor.cachedRelays?.relays.wireguard.portRanges ?? []
        )

        showInfo(with: String(
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
        ))

    case .wireGuardObfuscation:
        showInfo(with: NSLocalizedString(
            "VPN_SETTINGS_WIRE_GUARD_OBFUSCATION_GENERAL",
            tableName: "WireGuardObfuscation",
            value: """
            Obfuscation hides the WireGuard traffic inside another protocol. \
            It can be used to help circumvent censorship and other types of filtering, \
            where a plain WireGuard connect would be blocked.
            """,
            comment: ""
        ))

    case .wireGuardObfuscationPort:
        showInfo(with: NSLocalizedString(
            "VPN_SETTINGS_WIRE_GUARD_OBFUSCATION_PORT_GENERAL",
            tableName: "WireGuardObfuscation",
            value: "Which TCP port the UDP-over-TCP obfuscation protocol should connect to on the VPN server.",
            comment: ""
        ))

    case .quantumResistance:
        showInfo(with: NSLocalizedString(
            "VPN_SETTINGS_QUANTUM_RESISTANCE_GENERAL",
            tableName: "QuantumResistance",
            value: """
            This feature makes the WireGuard tunnel resistant to potential attacks from quantum computers.
            It does this by performing an extra key exchange using a quantum safe algorithm and mixing \
            the result into WireGuard’s regular encryption.
            This extra step uses approximately 500 kiB of traffic every time a new tunnel is established.
            """,
            comment: ""
        ))

    case .multihop:
        showInfo(with: NSLocalizedString(
            "MULTIHOP_INFORMATION_TEXT",
            tableName: "Multihop",
            value: """
            Multihop routes your traffic into one WireGuard server and out another, making it harder to trace.
            This results in increased latency but increases anonymity online.
            """,
            comment: ""
        ))
    case .daita:
        showInfo(with: NSLocalizedString(
            "DAITA_INFORMATION_TEXT",
            tableName: "DAITA",
            value: """
            DAITA (Defence against AI-guided Traffic Analysis) hides patterns in your encrypted VPN traffic. \
            If anyone is monitoring your connection, this makes it significantly harder for them to identify \
            what websites you are visiting. It does this by carefully adding network noise and making all \
            network packets the same size.
            Attention: Since this increases your total network traffic, be cautious if you have a limited data plan. \
            It can also negatively impact your network speed and battery usage.
            """,
            comment: ""
        ))
    default:
        assertionFailure("No matching InfoButtonItem")
    }
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

    func didAttemptToChangeDaitaSettings(_ settings: DAITASettings) -> DAITASettingsCompatibilityError? {
        interactor.evaluateDaitaSettingsCompatibility(settings)
    }

    // swiftlint:disable:next function_body_length
    func showPrompt(
        for item: VPNSettingsPromptAlertItem,
        onSave: @escaping () -> Void,
        onDiscard: @escaping () -> Void
    ) {
        let messageString = switch item {
        case .daitaSettingIncompatibleWithSinglehop:
            """
            DAITA isn’t available on the current server. After enabling, please go to the Switch \
            location view and select a location that supports DAITA.
            Attention: Since this increases your total network traffic, be cautious if you have a \
            limited data plan. It can also negatively impact your network speed and battery usage.
            """
        case .daitaSettingIncompatibleWithMultihop:
            """
            DAITA isn’t available on the current entry server. After enabling, please go to the Switch \
            location view and select an entry location that supports DAITA.
            Attention: Since this increases your total network traffic, be cautious if you have a \
            limited data plan. It can also negatively impact your network speed and battery usage.
            """
        }

        let presentation = AlertPresentation(
            id: "vpn-settings-content-blockers-alert",
            accessibilityIdentifier: .daitaPromptAlert,
            icon: .info,
            message: NSLocalizedString(
                "VPN_SETTINGS_VPN_DAITA_ENABLE_TEXT",
                tableName: "DAITA",
                value: messageString,
                comment: ""
            ),
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "VPN_SETTINGS_VPN_DAITA_OK_ACTION",
                        tableName: "DAITA",
                        value: "Enable anyway",
                        comment: ""
                    ),
                    style: .default,
                    accessibilityId: .daitaConfirmAlertEnableButton,
                    handler: { onSave() }
                ),
                AlertAction(
                    title: NSLocalizedString(
                        "VPN_SETTINGS_VPN_DAITA_CANCEL_ACTION",
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
