//
//  PreferencesDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 05/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class PreferencesDataSource: NSObject, UITableViewDataSource, UITableViewDelegate {
    private enum CellReuseIdentifiers: String, CaseIterable {
        case settingSwitch
        case settingStatus
        case dnsServer
        case editableDNSServer

        var reusableViewClass: AnyClass {
            switch self {
            case .settingSwitch:
                return SettingsSwitchCell.self
            case .settingStatus:
                return SettingsStatusCell.self
            case .dnsServer:
                return SettingsDNSServerAddressCell.self
            case .editableDNSServer:
                return SettingsDNSTextCell.self
            }
        }
    }

    private enum HeaderFooterReuseIdentifiers: String, CaseIterable {
        case customDNSFooter
        case spacer

        var reusableViewClass: AnyClass {
            switch self {
            case .customDNSFooter:
                return SettingsStaticTextFooterView.self
            case .spacer:
                return EmptyTableViewHeaderFooterView.self
            }
        }
    }

    private enum Section: Hashable {
        case mullvadDNS
        case customDNS
    }

    private enum Item: Hashable {
        case blockAdvertising
        case blockTracking
        case useCustomDNS
        case addDNSServer
        case dnsServer(_ index: Int)

        static func isDNSServerItem(_ item: Item) -> Bool {
            if case .dnsServer = item {
                return true
            } else {
                return false
            }
        }
    }

    private var isEditing = false
    private var snapshot = DataSourceSnapshot<Section, Item>()

    private(set) var viewModel = PreferencesViewModel()
    private(set) var viewModelBeforeEditing = PreferencesViewModel()

    weak var delegate: PreferencesDataSourceDelegate?

    weak var tableView: UITableView? {
        didSet {
            tableView?.dataSource = self
            tableView?.delegate = self

            registerCells()
        }
    }

    override init() {
        super.init()

        updateSnapshot()
    }

    func setEditing(_ editing: Bool) {
        guard isEditing != editing else { return }

        isEditing = editing

        if editing {
            viewModelBeforeEditing = viewModel
        } else {
            viewModel.endEditing()
        }

        updateSnapshot()
        tableView?.reloadData()

        if !editing && !viewModelBeforeEditing.compare(viewModel) {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    func update(from dnsSettings: DNSSettings) {
        let newViewModel = PreferencesViewModel(from: dnsSettings)

        if !viewModel.compare(newViewModel) {
            viewModel.merge(newViewModel)

            updateSnapshot()
            tableView?.reloadData()
        }
    }

    // MARK: - UITableViewDataSource

    func numberOfSections(in tableView: UITableView) -> Int {
        return snapshot.numberOfSections()
    }

    func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        guard let sectionIdentifier = snapshot.section(at: section) else { return 0 }

        return snapshot.numberOfItems(in: sectionIdentifier)
    }

    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let item = snapshot.itemForIndexPath(indexPath)!

        return dequeueCellForItem(item, in: tableView, at: indexPath)
    }

    func tableView(_ tableView: UITableView, canEditRowAt indexPath: IndexPath) -> Bool {
        // Disable swipe to delete when not editing the table view
        guard isEditing else { return false }

        let item = snapshot.itemForIndexPath(indexPath)

        switch item {
        case .dnsServer, .addDNSServer:
            return true
        default:
            return false
        }
    }

    func tableView(_ tableView: UITableView, commit editingStyle: UITableViewCell.EditingStyle, forRowAt indexPath: IndexPath) {
        let item = snapshot.itemForIndexPath(indexPath)

        if case .addDNSServer = item, editingStyle == .insert {
            addDNSServer()
        }

        if case .dnsServer(let serverIndex) = item, editingStyle == .delete {
            deleteDNSServer(at: serverIndex)
        }
    }

    func tableView(_ tableView: UITableView, canMoveRowAt indexPath: IndexPath) -> Bool {
        let item = snapshot.itemForIndexPath(indexPath)

        switch item {
        case .dnsServer:
            return true
        default:
            return false
        }
    }

    func tableView(_ tableView: UITableView, moveRowAt sourceIndexPath: IndexPath, to destinationIndexPath: IndexPath) {
        let sourceItem = snapshot.itemForIndexPath(sourceIndexPath)!
        let destinationItem = snapshot.itemForIndexPath(destinationIndexPath)!

        if case .dnsServer(let sourceIndex) = sourceItem, case .dnsServer(let destinationIndex) = destinationItem {
            viewModel.customDNSDomains.swapAt(sourceIndex, destinationIndex)
        }
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        return false
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        return tableView.dequeueReusableHeaderFooterView(withIdentifier: HeaderFooterReuseIdentifiers.spacer.rawValue)
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        let sectionIdentifier = snapshot.section(at: section)!

        switch sectionIdentifier {
        case .mullvadDNS:
            return nil

        case .customDNS:
            let reusableView = tableView.dequeueReusableHeaderFooterView(withIdentifier: HeaderFooterReuseIdentifiers.customDNSFooter.rawValue) as! SettingsStaticTextFooterView

            reusableView.titleLabel.text = NSLocalizedString(
                "CUSTOM_DNS_CELL_DISABLED_FOOTER_LABEL",
                tableName: "Preferences",
                value: "Disable Block Ads and Block trackers to activate this setting.",
                comment: ""
            )

            return reusableView
        }
    }

    func tableView(_ tableView: UITableView, heightForHeaderInSection section: Int) -> CGFloat {
        return UIMetrics.sectionSpacing
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        let sectionIdentifier = snapshot.section(at: section)!

        switch sectionIdentifier {
        case .mullvadDNS:
            return 0

        case .customDNS:
            if viewModel.canEnableCustomDNS {
                return 0
            } else {
                return UITableView.automaticDimension
            }
        }
    }

    func tableView(_ tableView: UITableView, editingStyleForRowAt indexPath: IndexPath) -> UITableViewCell.EditingStyle {
        let item = snapshot.itemForIndexPath(indexPath)

        switch item {
        case .dnsServer:
            return .delete
        case .addDNSServer:
            return .insert
        default:
            return .none
        }
    }

    func tableView(_ tableView: UITableView, targetIndexPathForMoveFromRowAt sourceIndexPath: IndexPath, toProposedIndexPath proposedDestinationIndexPath: IndexPath) -> IndexPath {
        guard let sectionIdentifier = snapshot.section(at: sourceIndexPath.section),
              case .customDNS = sectionIdentifier else { return sourceIndexPath }

        let items = snapshot.items(in: sectionIdentifier)

        let indexPathForFirstRow = items.first(where: Item.isDNSServerItem).flatMap { item in
            return snapshot.indexPathForItem(item, in: sectionIdentifier)
        }

        let indexPathForLastRow = items.last(where: Item.isDNSServerItem).flatMap { item in
            return snapshot.indexPathForItem(item, in: sectionIdentifier)
        }

        guard let indexPathForFirstRow = indexPathForFirstRow,
              let indexPathForLastRow = indexPathForLastRow else { return sourceIndexPath }

        if proposedDestinationIndexPath.section == sourceIndexPath.section {
            return min(max(proposedDestinationIndexPath, indexPathForFirstRow), indexPathForLastRow)
        } else {
            if proposedDestinationIndexPath.section > sourceIndexPath.section {
                return indexPathForLastRow
            } else {
                return indexPathForFirstRow
            }
        }
    }

    func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        let item = snapshot.itemForIndexPath(indexPath)

        if case .dnsServer = item, !isEditing {
            return 1
        } else {
            return 0
        }
    }

    // MARK: - Private

    private func registerCells() {
        CellReuseIdentifiers.allCases.forEach { enumCase in
            tableView?.register(enumCase.reusableViewClass, forCellReuseIdentifier: enumCase.rawValue)
        }

        HeaderFooterReuseIdentifiers.allCases.forEach { enumCase in
            tableView?.register(enumCase.reusableViewClass, forHeaderFooterViewReuseIdentifier: enumCase.rawValue)
        }
    }

    private func updateSnapshot() {
        var newSnapshot = DataSourceSnapshot<Section, Item>()
        newSnapshot.appendSections([.mullvadDNS, .customDNS])
        newSnapshot.appendItems([.blockAdvertising, .blockTracking], in: .mullvadDNS)
        newSnapshot.appendItems([.useCustomDNS], in: .customDNS)

        let dnsServerItems = viewModel.customDNSDomains.enumerated().map { i, _ in Item.dnsServer(i) }
        newSnapshot.appendItems(dnsServerItems, in: .customDNS)

        if isEditing {
            newSnapshot.appendItems([.addDNSServer], in: .customDNS)
        }

        snapshot = newSnapshot
    }

    private func dequeueCellForItem(_ item: Item, in tableView: UITableView, at indexPath: IndexPath) -> UITableViewCell {
        switch item {
        case .blockAdvertising:
            let titleLabel = NSLocalizedString(
                "BLOCK_ADS_CELL_LABEL",
                tableName: "Preferences",
                value: "Block ads",
                comment: ""
            )

            if isEditing {
                let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.settingSwitch.rawValue, for: indexPath) as! SettingsSwitchCell

                cell.titleLabel.text = titleLabel
                cell.accessibilityHint = nil
                cell.setOn(viewModel.blockAdvertising, animated: false)
                cell.action = { [weak self] isOn in
                    self?.setBlockAdvertising(isOn)
                }

                return cell
            } else {
                let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.settingStatus.rawValue, for: indexPath) as! SettingsStatusCell

                cell.titleLabel.text = titleLabel
                cell.isOn = viewModel.blockAdvertising

                return cell
            }

        case .blockTracking:
            let titleLabel = NSLocalizedString(
                "BLOCK_TRACKERS_CELL_LABEL",
                tableName: "Preferences",
                value: "Block trackers",
                comment: ""
            )

            if isEditing {
                let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.settingSwitch.rawValue, for: indexPath) as! SettingsSwitchCell

                cell.titleLabel.text = titleLabel
                cell.accessibilityHint = nil
                cell.setOn(viewModel.blockTracking, animated: false)
                cell.action = { [weak self] isOn in
                    self?.setBlockTracking(isOn)
                }

                return cell
            } else {
                let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.settingStatus.rawValue, for: indexPath) as! SettingsStatusCell

                cell.titleLabel.text = titleLabel
                cell.isOn = viewModel.blockTracking

                return cell
            }

        case .useCustomDNS:
            let titleLabel = NSLocalizedString(
                "CUSTOM_DNS_CELL_LABEL",
                tableName: "Preferences",
                value: "Use custom DNS",
                comment: ""
            )

            if isEditing {
                let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.settingSwitch.rawValue, for: indexPath) as! SettingsSwitchCell

                cell.titleLabel.text = titleLabel
                cell.setEnabled(viewModel.canEnableCustomDNS)

                if viewModel.canEnableCustomDNS {
                    cell.accessibilityHint = nil
                } else {
                    cell.accessibilityHint = NSLocalizedString(
                        "CUSTOM_DNS_CELL_DISABLED_ACCESSIBILITY_HINT",
                        tableName: "Preferences",
                        value: "Disable Block Ads and Block trackers to activate this setting.",
                        comment: ""
                    )
                }

                cell.setOn(viewModel.effectiveEnableCustomDNS, animated: false)
                cell.action = { [weak self] isOn in
                    self?.setEnableCustomDNS(isOn)
                }

                return cell
            } else {
                let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.settingStatus.rawValue, for: indexPath) as! SettingsStatusCell

                cell.titleLabel.text = titleLabel
                cell.isOn = viewModel.effectiveEnableCustomDNS

                return cell
            }

        case .addDNSServer:
            let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.editableDNSServer.rawValue, for: indexPath) as! SettingsDNSTextCell
            cell.textField.text = viewModel.dnsServerUserInput
            cell.isValidInput = viewModel.isValidDNSDomainInput(viewModel.dnsServerUserInput)

            cell.onTextChange = { [weak self] cell in
                guard let self = self else { return }

                let text = cell.textField.text ?? ""

                self.viewModel.dnsServerUserInput = text
                cell.isValidInput = self.viewModel.isValidDNSDomainInput(text)
            }

            cell.onReturnKey = { [weak self] cell in
                let text = cell.textField.text ?? ""

                if text.isEmpty {
                    cell.endEditing(false)
                } else {
                    self?.addDNSServer()
                }

            }

            return cell

        case .dnsServer(let serverIndex):
            let dnsServerAddress = viewModel.customDNSDomains[serverIndex]

            if isEditing {
                let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.editableDNSServer.rawValue, for: indexPath) as! SettingsDNSTextCell
                cell.textField.text = dnsServerAddress
                cell.isValidInput = viewModel.isValidDNSDomainInput(dnsServerAddress)

                cell.onTextChange = { [weak self] cell in
                    guard let self = self else { return }

                    let text = cell.textField.text ?? ""

                    self.viewModel.customDNSDomains[serverIndex] = text
                    cell.isValidInput = self.viewModel.isValidDNSDomainInput(text)
                }

                cell.onReturnKey = { cell in
                    cell.endEditing(false)
                }

                return cell
            } else {
                let cell = tableView.dequeueReusableCell(withIdentifier: CellReuseIdentifiers.dnsServer.rawValue, for: indexPath) as! SettingsDNSServerAddressCell

                cell.titleLabel.text = dnsServerAddress

                return cell
            }
        }
    }

    private func setBlockAdvertising(_ isEnabled: Bool) {
        let oldDataModel = viewModel

        viewModel.blockAdvertising = isEnabled

        updateCustomDNSFooterIfNeeded(oldDataModel: oldDataModel)
    }

    private func setBlockTracking(_ isEnabled: Bool) {
        let oldDataModel = viewModel

        viewModel.blockTracking = isEnabled

        updateCustomDNSFooterIfNeeded(oldDataModel: oldDataModel)
    }

    private func setEnableCustomDNS(_ isEnabled: Bool) {
        let oldDataModel = viewModel

        viewModel.enableCustomDNS = isEnabled

        updateCustomDNSFooterIfNeeded(oldDataModel: oldDataModel)
    }

    private func updateCustomDNSFooterIfNeeded(oldDataModel: PreferencesViewModel) {
        guard oldDataModel.canEnableCustomDNS != viewModel.canEnableCustomDNS else { return }

        tableView?.performBatchUpdates {
            let sectionIndex = snapshot.indexOfSection(.customDNS)!
            let indexPath = snapshot.indexPathForItem(.useCustomDNS, in: .customDNS)!

            // Reload "Use custom DNS" row
            tableView?.reloadRows(at: [indexPath], with: .none)

            // This call reloads the footer view
            _ = tableView?.footerView(forSection: sectionIndex)
        }
    }

    private func addDNSServer() {
        let ipAddress = AnyIPAddress(viewModel.dnsServerUserInput)
        let newServerIndex = viewModel.customDNSDomains.count

        let indexPathForAddServerItem = snapshot.indexPathForItem(.addDNSServer, in: .customDNS)!
        let addServerCell = tableView?.cellForRow(at: indexPathForAddServerItem) as? SettingsDNSTextCell

        if let ipAddress = ipAddress {
            viewModel.dnsServerUserInput = ""
            viewModel.customDNSDomains.append("\(ipAddress)")
            updateSnapshot()

            addServerCell?.textField.text = viewModel.dnsServerUserInput
            addServerCell?.isValidInput = viewModel.isValidDNSDomainInput(viewModel.dnsServerUserInput)

            let indexPathForNewServer = snapshot.indexPathForItem(.dnsServer(newServerIndex), in: .customDNS)!

            tableView?.performBatchUpdates({
                tableView?.insertRows(at: [indexPathForNewServer], with: .automatic)
            }, completion: { completed in
                if completed {
                    // Scroll to the text input field.
                    if let indexPath = self.snapshot.indexPathForItem(.addDNSServer, in: .customDNS) {
                        self.tableView?.scrollToRow(at: indexPath, at: .bottom, animated: true)
                    }
                }
            })
        } else {
            addServerCell?.isValidInput = false
        }
    }

    private func deleteDNSServer(at index: Int) {
        let indexPath = snapshot.indexPathForItem(.dnsServer(index), in: .customDNS)!

        viewModel.customDNSDomains.remove(at: index)
        updateSnapshot()

        tableView?.performBatchUpdates {
            tableView?.deleteRows(at: [indexPath], with: .automatic)
        }
    }

}
