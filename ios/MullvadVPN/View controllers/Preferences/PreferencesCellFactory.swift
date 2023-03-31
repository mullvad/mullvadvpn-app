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

    func didChangeDNSEntry(with identifier: UUID, inputString: String)
    func didChangeState(for item: PreferencesDataSource.Item, isOn: Bool)

    func addTrustedNetworkEntry(ssid: String?, beginEditing: Bool)
    func didChangeTrustedNetworkEntry(with identifier: UUID, newSsid: String)
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
        let cell = tableView.dequeueReusableCell(withIdentifier: item.cellIdentifier.rawValue, for: indexPath)

        configureCell(cell, item: item, indexPath: indexPath)

        return cell
    }

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
            cell.setOn(viewModel.blockMalware, animated: false)
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
            cell.setOn(viewModel.blockGambling, animated: false)
            cell.action = { [weak self] isOn in
                self?.delegate?.didChangeState(
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
                self?.delegate?.didChangeState(for: .useCustomDNS, isOn: isOn)
            }

        case .addDNSServer:
            guard let cell = cell as? SettingsAddEntryCell else { return }

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
            cell.isValidInput = dnsServerEntry.isValid

            cell.onTextChange = { [weak self] cell in
                let dnsAddress = cell.textField.text ?? ""

                self?.delegate?.didChangeDNSEntry(with: entryIdentifier, inputString: dnsAddress)

                cell.isValidInput = DNSServerEntry(address: dnsAddress).isValid // FIXME:
            }

            cell.onReturnKey = { cell in
                cell.endEditing(false)
            }

        case .addTrustedNetwork:
            guard let cell = cell as? SettingsAddEntryCell else { return }
            cell.titleLabel.text = NSLocalizedString(
                "ADD_TRUSTED_NETWORK_CELL",
                tableName: "Preferences",
                value: "Add trusted network",
                comment: ""
            )

            cell.action = { [weak self] in
                self?.delegate?.addTrustedNetworkEntry(ssid: nil, beginEditing: true)
            }

        case .addConnectedNetwork:
            guard let cell = cell as? AddConnectedNetworkCell else { return }

            let connectedNetwork = viewModel.connectedNetwork

            cell.connectedNetwork = connectedNetwork
            cell.action = { [weak self] in
                if let ssid = connectedNetwork?.ssid {
                    self?.delegate?.addTrustedNetworkEntry(ssid: ssid, beginEditing: false)
                }
            }

        case let .trustedNetwork(entryIdentifier):
            guard let cell = cell as? TrustedNetworkTextCell else { return }

            let trustedNetwork = viewModel.trustedNetwork(entryIdentifier: entryIdentifier)

            cell.textField.text = trustedNetwork?.ssid

            cell.onTextChange = { [weak self] cell in
                self?.delegate?.didChangeTrustedNetworkEntry(with: entryIdentifier, newSsid: cell.textField.text ?? "")
            }

            cell.onReturnKey = { cell in
                cell.endEditing(false)
            }

        case .useTrustedNetworks:
            guard let cell = cell as? SettingsSwitchCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "USE_TRUSTED_NETWORKS_CELL_LABEL",
                tableName: "Preferences",
                value: "Use trusted networks",
                comment: ""
            )
            cell.setEnabled(viewModel.hasValidTrustedNetworks)
            cell.setOn(viewModel.effectiveUseTrustedNetworks, animated: false)
            cell.accessibilityHint = nil
            cell.action = { [weak self] isOn in
                self?.delegate?.didChangeState(for: .useTrustedNetworks, isOn: isOn)
            }
        }
    }
}

extension PreferencesDataSource.Item {
    var cellIdentifier: PreferencesDataSource.CellReuseIdentifiers {
        switch self {
        case .blockAdvertising, .blockTracking, .blockMalware, .blockAdultContent, .blockGambling, .useCustomDNS:
            return .settingSwitch
        case .addDNSServer:
            return .addDNSServer
        case .dnsServer:
            return .dnsServer
        case .addConnectedNetwork:
            return .addConnectedNetwork
        case .addTrustedNetwork:
            return .addDNSServer
        case .trustedNetwork:
            return .trustedNetwork
        case .useTrustedNetworks:
            return .settingSwitch
        }
    }
}
