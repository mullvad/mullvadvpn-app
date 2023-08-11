//
//  PreferencesCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol PreferencesCellEventHandler {
    func addDNSEntry()
    func didChangeDNSEntry(with identifier: UUID, inputString: String) -> Bool
    func didChangeState(for item: PreferencesDataSource.Item, isOn: Bool)
    func showInfo(for item: PreferencesDataSource.InfoButtonItem)
    func addCustomPort(_ port: UInt16)
    func selectCustomPortEntry(_ port: UInt16) -> Bool
}

final class PreferencesCellFactory: CellFactoryProtocol {
    let tableView: UITableView
    var viewModel: PreferencesViewModel
    var delegate: PreferencesCellEventHandler?
    var isEditing = false

    init(tableView: UITableView, viewModel: PreferencesViewModel) {
        self.tableView = tableView
        self.viewModel = viewModel
    }

    func makeCell(for item: PreferencesDataSource.Item, indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: item.reuseIdentifier.rawValue, for: indexPath)

        configureCell(cell, item: item, indexPath: indexPath)

        return cell
    }

    // swiftlint:disable:next cyclomatic_complexity function_body_length
    func configureCell(_ cell: UITableViewCell, item: PreferencesDataSource.Item, indexPath: IndexPath) {
        switch item {
        case .blockAdvertising:
            guard let cell = cell as? SettingsSwitchCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "BLOCK_ADS_CELL_LABEL",
                tableName: "Preferences",
                value: "Block ads",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.applySubCellStyling()
            cell.setOn(viewModel.blockAdvertising, animated: false)
            cell.action = { [weak self] isOn in
                self?.delegate?.didChangeState(
                    for: .blockAdvertising,
                    isOn: isOn
                )
            }

        case .blockTracking:
            guard let cell = cell as? SettingsSwitchCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "BLOCK_TRACKERS_CELL_LABEL",
                tableName: "Preferences",
                value: "Block trackers",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.applySubCellStyling()
            cell.setOn(viewModel.blockTracking, animated: false)
            cell.action = { [weak self] isOn in
                self?.delegate?.didChangeState(
                    for: .blockTracking,
                    isOn: isOn
                )
            }

        case .blockMalware:
            guard let cell = cell as? SettingsSwitchCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "BLOCK_MALWARE_CELL_LABEL",
                tableName: "Preferences",
                value: "Block malware",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.applySubCellStyling()
            cell.setInfoButtonIsVisible(true)
            cell.setOn(viewModel.blockMalware, animated: false)
            cell.infoButtonHandler = { [weak self] in
                self?.delegate?.showInfo(for: .blockMalware)
            }
            cell.action = { [weak self] isOn in
                self?.delegate?.didChangeState(for: .blockMalware, isOn: isOn)
            }

        case .blockAdultContent:
            guard let cell = cell as? SettingsSwitchCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "BLOCK_ADULT_CELL_LABEL",
                tableName: "Preferences",
                value: "Block adult content",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.applySubCellStyling()
            cell.setOn(viewModel.blockAdultContent, animated: false)
            cell.action = { [weak self] isOn in
                self?.delegate?.didChangeState(
                    for: .blockAdultContent,
                    isOn: isOn
                )
            }

        case .blockGambling:
            guard let cell = cell as? SettingsSwitchCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "BLOCK_GAMBLING_CELL_LABEL",
                tableName: "Preferences",
                value: "Block gambling",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.applySubCellStyling()
            cell.setOn(viewModel.blockGambling, animated: false)
            cell.action = { [weak self] isOn in
                self?.delegate?.didChangeState(
                    for: .blockGambling,
                    isOn: isOn
                )
            }

        case let .wireGuardPort(port):
            guard let cell = cell as? SelectableSettingsCell else { return }

            var portString = NSLocalizedString(
                "WIRE_GUARD_PORT_CELL_LABEL",
                tableName: "Preferences",
                value: "Automatic",
                comment: ""
            )
            if let port {
                portString = String(port)
            }

            cell.titleLabel.text = portString
            cell.accessibilityHint = nil
            cell.applySubCellStyling()

        case .wireGuardCustomPort:
            guard let cell = cell as? SettingsInputCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "WIRE_GUARD_CUSTOM_PORT_CELL_LABEL",
                tableName: "Preferences",
                value: "Custom",
                comment: ""
            )
            cell.textField.placeholder = NSLocalizedString(
                "WIRE_GUARD_CUSTOM_PORT_CELL_INPUT_PLACEHOLDER",
                tableName: "Preferences",
                value: "Port",
                comment: ""
            )

            cell.accessibilityHint = nil
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

        case .useCustomDNS:
            guard let cell = cell as? SettingsSwitchCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "CUSTOM_DNS_CELL_LABEL",
                tableName: "Preferences",
                value: "Use custom DNS",
                comment: ""
            )
            cell.setEnabled(viewModel.customDNSPrecondition == .satisfied)
            cell.setOn(viewModel.effectiveEnableCustomDNS, animated: false)
            cell.accessibilityHint = viewModel.customDNSPrecondition
                .localizedDescription(isEditing: isEditing)
            cell.action = { [weak self] isOn in
                self?.delegate?.didChangeState(for: .useCustomDNS, isOn: isOn)
            }

        case .addDNSServer:
            guard let cell = cell as? SettingsAddDNSEntryCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "ADD_CUSTOM_DNS_SERVER_CELL_LABEL",
                tableName: "Preferences",
                value: "Add a server",
                comment: ""
            )
            cell.action = { [weak self] in
                self?.delegate?.addDNSEntry()
            }

        case let .dnsServer(entryIdentifier):
            guard let cell = cell as? SettingsDNSTextCell else { return }

            let dnsServerEntry = viewModel.dnsEntry(entryIdentifier: entryIdentifier)!

            cell.textField.text = dnsServerEntry.address
            cell.isValidInput = dsnEntryIsValid(identifier: entryIdentifier, cell: cell)

            cell.onTextChange = { [weak self] cell in
                cell.isValidInput = self?
                    .dsnEntryIsValid(identifier: entryIdentifier, cell: cell) ?? false
            }

            cell.onReturnKey = { cell in
                cell.endEditing(false)
            }

        case .dnsServerInfo:
            guard let cell = cell as? SettingsDNSInfoCell else { return }

            cell.titleLabel.attributedText = viewModel.customDNSPrecondition.attributedLocalizedDescription(
                isEditing: isEditing,
                preferredFont: .systemFont(ofSize: 14)
            )
        case .wireGuardObfuscationAutomatic:
            guard let cell = cell as? SelectableSettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "WIRE_GUARD_OBFUSCATION_AUTOMATIC_LABEL",
                tableName: "Preferences",
                value: "Automatic",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.applySubCellStyling()
        case .wireGuardObfuscationOn:
            guard let cell = cell as? SelectableSettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "WIRE_GUARD_OBFUSCATION_ON_LABEL",
                tableName: "Preferences",
                value: "On (UDP-over-TCP)",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.applySubCellStyling()
        case .wireGuardObfuscationOff:
            guard let cell = cell as? SelectableSettingsCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "WIRE_GUARD_OBFUSCATION_OFF_LABEL",
                tableName: "Preferences",
                value: "Off",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.applySubCellStyling()

        case let .wireGuardObfuscationPort(port):
            guard let cell = cell as? SelectableSettingsCell else { return }

            let portValue = port == 0 ? "Automatic" : "\(port)"

            cell.titleLabel.text = NSLocalizedString(
                "WIRE_GUARD_OBFUSCATION_PORT_LABEL",
                tableName: "Preferences",
                value: portValue,
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.applySubCellStyling()
        }
    }

    private func dsnEntryIsValid(identifier: UUID, cell: SettingsDNSTextCell) -> Bool {
        delegate?.didChangeDNSEntry(
            with: identifier,
            inputString: cell.textField.text ?? ""
        ) ?? false
    }
}
