//
//  VPNSettingsCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

protocol VPNSettingsCellEventHandler {
    func showInfo(for button: VPNSettingsInfoButtonItem)
    func addCustomPort(_ port: UInt16)
    func selectCustomPortEntry(_ port: UInt16) -> Bool
    func selectObfuscationState(_ state: WireGuardObfuscationState)
}

final class VPNSettingsCellFactory: CellFactoryProtocol {
    let tableView: UITableView
    var viewModel: VPNSettingsViewModel
    var delegate: VPNSettingsCellEventHandler?
    var isEditing = false

    init(tableView: UITableView, viewModel: VPNSettingsViewModel) {
        self.tableView = tableView
        self.viewModel = viewModel
    }

    func makeCell(for item: VPNSettingsDataSource.Item, indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: item.reuseIdentifier.rawValue, for: indexPath)

        configureCell(cell, item: item, indexPath: indexPath)

        return cell
    }

    // swiftlint:disable:next cyclomatic_complexity function_body_length
    func configureCell(_ cell: UITableViewCell, item: VPNSettingsDataSource.Item, indexPath: IndexPath) {
        switch item {
        case .dnsSettings:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "DNS_SETTINGS_CELL_LABEL",
                tableName: "VPNSettings",
                value: "DNS settings",
                comment: ""
            )

            cell.disclosureType = .chevron
            cell.accessibilityIdentifier = item.accessibilityIdentifier

        case .ipOverrides:
            guard let cell = cell as? SettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "IP_OVERRIDE_CELL_LABEL",
                tableName: "VPNSettings",
                value: "Server IP override",
                comment: ""
            )

            cell.disclosureType = .chevron
            cell.accessibilityIdentifier = item.accessibilityIdentifier

        case let .wireGuardPort(port):
            guard let cell = cell as? SelectableSettingsCell else { return }

            var portString = NSLocalizedString(
                "WIRE_GUARD_PORT_CELL_LABEL",
                tableName: "VPNSettings",
                value: "Automatic",
                comment: ""
            )
            if let port {
                portString = String(port)
            }

            cell.titleLabel.text = portString
            cell.accessibilityIdentifier = "\(item.accessibilityIdentifier.rawValue) (\(portString))"
            cell.applySubCellStyling()

        case .wireGuardCustomPort:
            guard let cell = cell as? SettingsInputCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "WIRE_GUARD_CUSTOM_PORT_CELL_LABEL",
                tableName: "VPNSettings",
                value: "Custom",
                comment: ""
            )
            cell.textField.placeholder = NSLocalizedString(
                "WIRE_GUARD_CUSTOM_PORT_CELL_INPUT_PLACEHOLDER",
                tableName: "VPNSettings",
                value: "Port",
                comment: ""
            )

            cell.textField.accessibilityIdentifier = .customWireGuardPortTextField
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.applySubCellStyling()

            cell.inputDidChange = { [weak self] text in
                let port = UInt16(text) ?? UInt16()
                cell.isValidInput = self?.delegate?.selectCustomPortEntry(port) ?? false
            }
            cell.inputWasConfirmed = { [weak self] text in
                if let port = UInt16(text), cell.isValidInput {
                    self?.delegate?.addCustomPort(port)
                }
            }

            if let port = viewModel.customWireGuardPort {
                cell.textField.text = String(port)

                // Only update validity if input is invalid. Otherwise the textcolor will be wrong
                // (active text field color rather than the expected inactive color).
                let isValidInput = delegate?.selectCustomPortEntry(port) ?? false
                if !isValidInput {
                    cell.isValidInput = false
                }
            }

        case .wireGuardObfuscationAutomatic:
            guard let cell = cell as? SelectableSettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "WIRE_GUARD_OBFUSCATION_AUTOMATIC_LABEL",
                tableName: "VPNSettings",
                value: "Automatic",
                comment: ""
            )
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.applySubCellStyling()

        case .wireGuardObfuscationOn:
            guard let cell = cell as? SelectableSettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "WIRE_GUARD_OBFUSCATION_ON_LABEL",
                tableName: "VPNSettings",
                value: "On (UDP-over-TCP)",
                comment: ""
            )
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.applySubCellStyling()
        case .wireGuardObfuscationOff:
            guard let cell = cell as? SelectableSettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "WIRE_GUARD_OBFUSCATION_OFF_LABEL",
                tableName: "VPNSettings",
                value: "Off",
                comment: ""
            )
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.applySubCellStyling()

        case let .wireGuardObfuscationPort(port):
            guard let cell = cell as? SelectableSettingsCell else { return }

            let portString = port == 0 ? "Automatic" : "\(port)"
            cell.titleLabel.text = NSLocalizedString(
                "WIRE_GUARD_OBFUSCATION_PORT_LABEL",
                tableName: "VPNSettings",
                value: portString,
                comment: ""
            )
            cell.accessibilityIdentifier = "\(item.accessibilityIdentifier.rawValue) (\(portString))"
            cell.applySubCellStyling()

        case .quantumResistanceAutomatic:
            guard let cell = cell as? SelectableSettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "QUANTUM_RESISTANCE_AUTOMATIC_LABEL",
                tableName: "VPNSettings",
                value: "Automatic",
                comment: ""
            )
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.applySubCellStyling()

        case .quantumResistanceOn:
            guard let cell = cell as? SelectableSettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "QUANTUM_RESISTANCE_ON_LABEL",
                tableName: "VPNSettings",
                value: "On",
                comment: ""
            )
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.applySubCellStyling()
        case .quantumResistanceOff:
            guard let cell = cell as? SelectableSettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "QUANTUM_RESISTANCE_OFF_LABEL",
                tableName: "VPNSettings",
                value: "Off",
                comment: ""
            )
            cell.accessibilityIdentifier = item.accessibilityIdentifier
            cell.applySubCellStyling()
        }
    }
}
