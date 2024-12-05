//
//  CustomDNSCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-11-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

protocol CustomDNSCellEventHandler {
    func addDNSEntry()
    func didChangeDNSEntry(with identifier: UUID, inputString: String) -> Bool
    func didChangeState(for preference: CustomDNSDataSource.Item, isOn: Bool)
    func showInfo(for button: VPNSettingsInfoButtonItem)
}

final class CustomDNSCellFactory: CellFactoryProtocol {
    let tableView: UITableView
    var viewModel: VPNSettingsViewModel
    var delegate: CustomDNSCellEventHandler?
    var isEditing = false

    init(tableView: UITableView, viewModel: VPNSettingsViewModel) {
        self.tableView = tableView
        self.viewModel = viewModel
    }

    func makeCell(for item: CustomDNSDataSource.Item, indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: item.reuseIdentifier.rawValue, for: indexPath)

        configureCell(cell, item: item, indexPath: indexPath)

        return cell
    }

    func configure(
        _ cell: UITableViewCell,
        toggleSetting: Bool,
        title: String,
        for preference: CustomDNSDataSource.Item
    ) {
        guard let cell = cell as? SettingsSwitchCell else { return }

        cell.titleLabel.text = title
        cell.setAccessibilityIdentifier(preference.accessibilityIdentifier)
        cell.applySubCellStyling()
        cell.setOn(toggleSetting, animated: true)
        cell.action = { [weak self] isOn in
            self?.delegate?.didChangeState(
                for: preference,
                isOn: isOn
            )
        }
    }

    // swiftlint:disable:next function_body_length
    func configureCell(_ cell: UITableViewCell, item: CustomDNSDataSource.Item, indexPath: IndexPath) {
        switch item {
        case .blockAll:
            let localizedString = NSLocalizedString(
                "BLOCK_ALL_CELL_LABEL",
                tableName: "VPNSettings",
                value: "All",
                comment: ""
            )

            configure(
                cell,
                toggleSetting: viewModel.blockAll,
                title: localizedString,
                for: .blockAll
            )

        case .blockAdvertising:
            let localizedString = NSLocalizedString(
                "BLOCK_ADS_CELL_LABEL",
                tableName: "VPNSettings",
                value: "Ads",
                comment: ""
            )

            configure(
                cell,
                toggleSetting: viewModel.blockAdvertising,
                title: localizedString,
                for: .blockAdvertising
            )

        case .blockTracking:
            let localizedString = NSLocalizedString(
                "BLOCK_TRACKERS_CELL_LABEL",
                tableName: "VPNSettings",
                value: "Trackers",
                comment: ""
            )
            configure(
                cell,
                toggleSetting: viewModel.blockTracking,
                title: localizedString,
                for: .blockTracking
            )

        case .blockMalware:
            guard let cell = cell as? SettingsSwitchCell else { return }

            let localizedString = NSLocalizedString(
                "BLOCK_MALWARE_CELL_LABEL",
                tableName: "VPNSettings",
                value: "Malware",
                comment: ""
            )
            configure(
                cell,
                toggleSetting: viewModel.blockMalware,
                title: localizedString,
                for: .blockMalware
            )
            cell.infoButtonHandler = { [weak self] in
                self?.delegate?.showInfo(for: .blockMalware)
            }

        case .blockAdultContent:
            let localizedString = NSLocalizedString(
                "BLOCK_ADULT_CELL_LABEL",
                tableName: "VPNSettings",
                value: "Adult content",
                comment: ""
            )
            configure(
                cell,
                toggleSetting: viewModel.blockAdultContent,
                title: localizedString,
                for: .blockAdultContent
            )

        case .blockGambling:
            let localizedString = NSLocalizedString(
                "BLOCK_GAMBLING_CELL_LABEL",
                tableName: "VPNSettings",
                value: "Gambling",
                comment: ""
            )
            configure(
                cell,
                toggleSetting: viewModel.blockGambling,
                title: localizedString,
                for: .blockGambling
            )

        case .blockSocialMedia:
            let localizedString = NSLocalizedString(
                "BLOCK_SOCIAL_MEDIA_CELL_LABEL",
                tableName: "VPNSettings",
                value: "Social media",
                comment: ""
            )
            configure(
                cell,
                toggleSetting: viewModel.blockSocialMedia,
                title: localizedString,
                for: .blockSocialMedia
            )

        case .useCustomDNS:
            guard let cell = cell as? SettingsSwitchCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "CUSTOM_DNS_CELL_LABEL",
                tableName: "VPNSettings",
                value: "Use custom DNS server",
                comment: ""
            )
            cell.setSwitchEnabled(viewModel.customDNSPrecondition == .satisfied)
            cell.setOn(viewModel.effectiveEnableCustomDNS, animated: false)
            cell.accessibilityHint = viewModel.customDNSPrecondition
                .localizedDescription(isEditing: isEditing)
            cell.setAccessibilityIdentifier(.dnsSettingsUseCustomDNSCell)
            cell.action = { [weak self] isOn in
                self?.delegate?.didChangeState(for: .useCustomDNS, isOn: isOn)
            }

        case .addDNSServer:
            guard let cell = cell as? SettingsAddDNSEntryCell else { return }

            cell.titleLabel.text = NSLocalizedString(
                "ADD_CUSTOM_DNS_SERVER_CELL_LABEL",
                tableName: "VPNSettings",
                value: "Add a server",
                comment: ""
            )
            cell.setAccessibilityIdentifier(.dnsSettingsAddServerCell)
            cell.tapAction = { [weak self] in
                self?.delegate?.addDNSEntry()
            }

        case let .dnsServer(entryIdentifier):
            guard let cell = cell as? SettingsDNSTextCell else { return }

            let dnsServerEntry = viewModel.dnsEntry(entryIdentifier: entryIdentifier)!

            cell.textField.text = dnsServerEntry.address
            cell.isValidInput = dnsEntryIsValid(identifier: entryIdentifier, cell: cell)
            cell.accessibilityIdentifier = "\(item.accessibilityIdentifier) (\(entryIdentifier))"

            cell.onTextChange = { [weak self] cell in
                cell.isValidInput = self?
                    .dnsEntryIsValid(identifier: entryIdentifier, cell: cell) ?? false
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
        }
    }

    private func dnsEntryIsValid(identifier: UUID, cell: SettingsDNSTextCell) -> Bool {
        delegate?.didChangeDNSEntry(
            with: identifier,
            inputString: cell.textField.text ?? ""
        ) ?? false
    }
}
