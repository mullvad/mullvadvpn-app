//
//  PreferencesCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol PreferencesCellFactoryDelegate: AnyObject {
    func preferencesCellFactoryDidChangeState(for item: PreferencesDataSource.Item, isOn: Bool)
    func preferencesCellFactoryShouldAddDnsEntry()
    func preferencesCellFactoryShouldHandleDnsChange(
        entryIdentifier: UUID,
        cell: SettingsDNSTextCell
    )
}

final class PreferencesCellFactory: CellFactoryProtocol {
    let tableView: UITableView
    var viewModel: PreferencesViewModel
    weak var delegate: PreferencesCellFactoryDelegate?
    var isEditing = false

    init(tableView: UITableView, viewModel: PreferencesViewModel) {
        self.tableView = tableView
        self.viewModel = viewModel
    }

    func makeCell(for item: PreferencesDataSource.Item, indexPath: IndexPath) -> UITableViewCell {
        let cell: UITableViewCell

        switch item {
        case .addDNSServer:
            cell = tableView.dequeueReusableCell(
                withIdentifier: PreferencesDataSource.CellReuseIdentifiers.addDNSServer.rawValue,
                for: indexPath
            )
        case .dnsServer:
            cell = tableView.dequeueReusableCell(
                withIdentifier: PreferencesDataSource.CellReuseIdentifiers.dnsServer.rawValue,
                for: indexPath
            )
        default:
            cell = tableView.dequeueReusableCell(
                withIdentifier: PreferencesDataSource.CellReuseIdentifiers.settingSwitch.rawValue,
                for: indexPath
            )
        }

        configureCell(cell, item: item, indexPath: indexPath)

        return cell
    }

    func configureCell(
        _ cell: UITableViewCell,
        item: PreferencesDataSource.Item,
        indexPath: IndexPath
    ) {
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
            cell.setOn(viewModel.blockAdvertising, animated: false)
            cell.action = { [weak self] isOn in
                self?.delegate?.preferencesCellFactoryDidChangeState(
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
            cell.setOn(viewModel.blockTracking, animated: false)
            cell.action = { [weak self] isOn in
                self?.delegate?.preferencesCellFactoryDidChangeState(
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
            cell.setOn(viewModel.blockMalware, animated: false)
            cell.action = { [weak self] isOn in
                self?.delegate?.preferencesCellFactoryDidChangeState(for: .blockMalware, isOn: isOn)
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
            cell.setOn(viewModel.blockAdultContent, animated: false)
            cell.action = { [weak self] isOn in
                self?.delegate?.preferencesCellFactoryDidChangeState(
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
            cell.setOn(viewModel.blockGambling, animated: false)
            cell.action = { [weak self] isOn in
                self?.delegate?.preferencesCellFactoryDidChangeState(
                    for: .blockGambling,
                    isOn: isOn
                )
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
                self?.delegate?.preferencesCellFactoryDidChangeState(for: .useCustomDNS, isOn: isOn)
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
                self?.delegate?.preferencesCellFactoryShouldAddDnsEntry()
            }

        case let .dnsServer(entryIdentifier):
            guard let cell = cell as? SettingsDNSTextCell else { return }

            let dnsServerEntry = viewModel.dnsEntry(entryIdentifier: entryIdentifier)!

            cell.textField.text = dnsServerEntry.address
            cell.isValidInput = viewModel.validateDNSDomainUserInput(dnsServerEntry.address)

            cell.onTextChange = { [weak self] cell in
                self?.delegate?.preferencesCellFactoryShouldHandleDnsChange(
                    entryIdentifier: entryIdentifier,
                    cell: cell
                )
            }

            cell.onReturnKey = { cell in
                cell.endEditing(false)
            }
        }
    }
}
