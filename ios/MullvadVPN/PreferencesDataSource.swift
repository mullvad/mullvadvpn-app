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
        case dnsServer
        case addDNSServer

        var reusableViewClass: AnyClass {
            switch self {
            case .settingSwitch:
                return SettingsSwitchCell.self
            case .dnsServer:
                return SettingsDNSTextCell.self
            case .addDNSServer:
                return SettingsAddDNSEntryCell.self
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

    private enum Section: String, Hashable {
        case mullvadDNS
        case customDNS
    }

    private enum Item: Hashable {
        case blockAdvertising
        case blockTracking
        case blockMalware
        case blockAdultContent
        case blockGambling
        case useCustomDNS
        case addDNSServer
        case dnsServer(_ uniqueID: UUID)

        var accessibilityIdentifier: String {
            switch self {
            case .blockAdvertising:
                return "blockAdvertising"
            case .blockTracking:
                return "blockTracking"
            case .blockMalware:
                return "blockMalware"
            case .blockGambling:
                return "blockGambling"
            case .blockAdultContent:
                return "blockAdultContent"
            case .useCustomDNS:
                return "useCustomDNS"
            case .addDNSServer:
                return "addDNSServer"
            case let .dnsServer(uuid):
                return "dnsServer(\(uuid.uuidString))"
            }
        }

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

            registerClasses()
        }
    }

    override init() {
        super.init()

        updateSnapshot()
    }

    func setEditing(_ editing: Bool, animated: Bool) {
        guard isEditing != editing else { return }

        let oldSnapshot = snapshot
        let oldDNSDomains = viewModel.customDNSDomains

        isEditing = editing

        if editing {
            viewModelBeforeEditing = viewModel
        } else {
            viewModel.sanitizeCustomDNSEntries()
        }

        updateSnapshot()

        // Reconfigure cells for items with corresponding DNS entries that were changed during
        // sanitization.
        let itemsToReload: [Item] = oldDNSDomains.filter { oldDNSEntry in
            guard let newDNSEntry = viewModel.dnsEntry(entryIdentifier: oldDNSEntry.identifier)
            else { return false }

            return newDNSEntry.address != oldDNSEntry.address
        }.map { dnsEntry in
            return .dnsServer(dnsEntry.identifier)
        }

        snapshot.reconfigureItems(itemsToReload)

        if animated {
            let diffResult = oldSnapshot.difference(snapshot)
            if let tableView = tableView {
                diffResult.apply(to: tableView, animateDifferences: animated)
                reloadCustomDNSFooter()
            }
        } else {
            tableView?.reloadData()
        }

        if !editing, viewModelBeforeEditing != viewModel {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    func update(from dnsSettings: DNSSettings) {
        let newViewModel = PreferencesViewModel(from: dnsSettings)
        let mergedViewModel = viewModel.merged(newViewModel)

        if viewModel != mergedViewModel {
            viewModel = mergedViewModel
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

        return snapshot.numberOfItems(in: sectionIdentifier) ?? 0
    }

    func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let item = snapshot.itemForIndexPath(indexPath)!
        let cell = dequeueCellForItem(item, in: tableView, at: indexPath)

        let section = snapshot.section(at: indexPath.section)!
        cell.accessibilityIdentifier = "\(section.rawValue).\(item.accessibilityIdentifier)"

        return cell
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

    func tableView(
        _ tableView: UITableView,
        commit editingStyle: UITableViewCell.EditingStyle,
        forRowAt indexPath: IndexPath
    ) {
        let item = snapshot.itemForIndexPath(indexPath)

        if case .addDNSServer = item, editingStyle == .insert {
            addDNSServerEntry()
        }

        if case let .dnsServer(entryIdentifier) = item, editingStyle == .delete {
            deleteDNSServerEntry(entryIdentifier: entryIdentifier)
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

    func tableView(
        _ tableView: UITableView,
        moveRowAt sourceIndexPath: IndexPath,
        to destinationIndexPath: IndexPath
    ) {
        let sourceItem = snapshot.itemForIndexPath(sourceIndexPath)!
        let destinationItem = snapshot.itemForIndexPath(destinationIndexPath)!

        guard case let .dnsServer(sourceIdentifier) = sourceItem,
              case let .dnsServer(targetIdentifier) = destinationItem,
              let sourceIndex = viewModel.indexOfDNSEntry(entryIdentifier: sourceIdentifier),
              let destinationIndex = viewModel.indexOfDNSEntry(entryIdentifier: targetIdentifier)
        else { return }

        let removedEntry = viewModel.customDNSDomains.remove(at: sourceIndex)
        viewModel.customDNSDomains.insert(removedEntry, at: destinationIndex)

        updateSnapshot()
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        return false
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        return tableView.dequeueReusableHeaderFooterView(
            withIdentifier: HeaderFooterReuseIdentifiers.spacer.rawValue
        )
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        let sectionIdentifier = snapshot.section(at: section)!

        switch sectionIdentifier {
        case .mullvadDNS:
            return nil

        case .customDNS:
            let reusableView = tableView
                .dequeueReusableHeaderFooterView(
                    withIdentifier: HeaderFooterReuseIdentifiers
                        .customDNSFooter.rawValue
                ) as! SettingsStaticTextFooterView
            configureFooterView(reusableView)
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
            switch viewModel.customDNSPrecondition {
            case .satisfied:
                return 0
            case .conflictsWithOtherSettings, .emptyDNSDomains:
                return UITableView.automaticDimension
            }
        }
    }

    func tableView(
        _ tableView: UITableView,
        editingStyleForRowAt indexPath: IndexPath
    ) -> UITableViewCell.EditingStyle {
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

    func tableView(
        _ tableView: UITableView,
        targetIndexPathForMoveFromRowAt sourceIndexPath: IndexPath,
        toProposedIndexPath proposedDestinationIndexPath: IndexPath
    ) -> IndexPath {
        guard let sectionIdentifier = snapshot.section(at: sourceIndexPath.section),
              case .customDNS = sectionIdentifier else { return sourceIndexPath }

        let items = snapshot.items(in: sectionIdentifier)

        let indexPathForFirstRow = items.first(where: Item.isDNSServerItem).flatMap { item in
            return snapshot.indexPathForItem(item)
        }

        let indexPathForLastRow = items.last(where: Item.isDNSServerItem).flatMap { item in
            return snapshot.indexPathForItem(item)
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

    // MARK: - Private

    private func registerClasses() {
        CellReuseIdentifiers.allCases.forEach { enumCase in
            tableView?.register(
                enumCase.reusableViewClass,
                forCellReuseIdentifier: enumCase.rawValue
            )
        }

        HeaderFooterReuseIdentifiers.allCases.forEach { enumCase in
            tableView?.register(
                enumCase.reusableViewClass,
                forHeaderFooterViewReuseIdentifier: enumCase.rawValue
            )
        }
    }

    private func updateSnapshot() {
        var newSnapshot = DataSourceSnapshot<Section, Item>()
        newSnapshot.appendSections([.mullvadDNS, .customDNS])
        newSnapshot.appendItems(
            [.blockAdvertising, .blockTracking, .blockMalware, .blockAdultContent, .blockGambling],
            in: .mullvadDNS
        )
        newSnapshot.appendItems([.useCustomDNS], in: .customDNS)

        let dnsServerItems = viewModel.customDNSDomains.map { entry in
            return Item.dnsServer(entry.identifier)
        }
        newSnapshot.appendItems(dnsServerItems, in: .customDNS)

        if isEditing, viewModel.customDNSDomains.count < DNSSettings.maxAllowedCustomDNSDomains {
            newSnapshot.appendItems([.addDNSServer], in: .customDNS)
        }

        snapshot = newSnapshot
    }

    private func dequeueCellForItem(
        _ item: Item,
        in tableView: UITableView,
        at indexPath: IndexPath
    ) -> UITableViewCell {
        switch item {
        case .blockAdvertising:
            let cell = tableView.dequeueReusableCell(
                withIdentifier: CellReuseIdentifiers.settingSwitch.rawValue,
                for: indexPath
            ) as! SettingsSwitchCell

            cell.titleLabel.text = NSLocalizedString(
                "BLOCK_ADS_CELL_LABEL",
                tableName: "Preferences",
                value: "Block ads",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.setOn(viewModel.blockAdvertising, animated: false)
            cell.action = { [weak self] isOn in
                self?.setBlockAdvertising(isOn)
            }

            return cell

        case .blockTracking:
            let cell = tableView.dequeueReusableCell(
                withIdentifier: CellReuseIdentifiers.settingSwitch.rawValue,
                for: indexPath
            ) as! SettingsSwitchCell

            cell.titleLabel.text = NSLocalizedString(
                "BLOCK_TRACKERS_CELL_LABEL",
                tableName: "Preferences",
                value: "Block trackers",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.setOn(viewModel.blockTracking, animated: false)
            cell.action = { [weak self] isOn in
                self?.setBlockTracking(isOn)
            }

            return cell

        case .blockMalware:
            let cell = tableView.dequeueReusableCell(
                withIdentifier: CellReuseIdentifiers.settingSwitch.rawValue,
                for: indexPath
            ) as! SettingsSwitchCell

            cell.titleLabel.text = NSLocalizedString(
                "BLOCK_MALWARE_CELL_LABEL",
                tableName: "Preferences",
                value: "Block malware",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.setOn(viewModel.blockMalware, animated: false)
            cell.action = { [weak self] isOn in
                self?.setBlockMalware(isOn)
            }

            return cell

        case .blockAdultContent:
            let cell = tableView.dequeueReusableCell(
                withIdentifier: CellReuseIdentifiers.settingSwitch.rawValue,
                for: indexPath
            ) as! SettingsSwitchCell

            cell.titleLabel.text = NSLocalizedString(
                "BLOCK_ADULT_CELL_LABEL",
                tableName: "Preferences",
                value: "Block adult content",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.setOn(viewModel.blockAdultContent, animated: false)
            cell.action = { [weak self] isOn in
                self?.setBlockAdultContent(isOn)
            }

            return cell

        case .blockGambling:
            let cell = tableView.dequeueReusableCell(
                withIdentifier: CellReuseIdentifiers.settingSwitch.rawValue,
                for: indexPath
            ) as! SettingsSwitchCell

            cell.titleLabel.text = NSLocalizedString(
                "BLOCK_GAMBLING_CELL_LABEL",
                tableName: "Preferences",
                value: "Block gambling",
                comment: ""
            )
            cell.accessibilityHint = nil
            cell.setOn(viewModel.blockGambling, animated: false)
            cell.action = { [weak self] isOn in
                self?.setBlockGambling(isOn)
            }

            return cell

        case .useCustomDNS:
            let cell = tableView.dequeueReusableCell(
                withIdentifier: CellReuseIdentifiers.settingSwitch.rawValue,
                for: indexPath
            ) as! SettingsSwitchCell

            cell.titleLabel.text = NSLocalizedString(
                "CUSTOM_DNS_CELL_LABEL",
                tableName: "Preferences",
                value: "Use custom DNS",
                comment: ""
            )
            cell.setEnabled(viewModel.customDNSPrecondition == .satisfied)
            cell.setOn(viewModel.effectiveEnableCustomDNS, animated: false)
            cell.action = { [weak self] isOn in
                self?.setEnableCustomDNS(isOn)
            }

            cell.accessibilityHint = viewModel.customDNSPrecondition
                .localizedDescription(isEditing: isEditing)

            return cell

        case .addDNSServer:
            let cell = tableView.dequeueReusableCell(
                withIdentifier: CellReuseIdentifiers.addDNSServer.rawValue,
                for: indexPath
            ) as! SettingsAddDNSEntryCell
            cell.titleLabel.text = NSLocalizedString(
                "ADD_CUSTOM_DNS_SERVER_CELL_LABEL",
                tableName: "Preferences",
                value: "Add a server",
                comment: ""
            )

            cell.actionHandler = { [weak self] cell in
                self?.addDNSServerEntry()
            }

            return cell

        case let .dnsServer(entryIdentifier):
            let dnsServerEntry = viewModel.dnsEntry(entryIdentifier: entryIdentifier)!

            let cell = tableView.dequeueReusableCell(
                withIdentifier: CellReuseIdentifiers.dnsServer.rawValue,
                for: indexPath
            ) as! SettingsDNSTextCell
            cell.textField.text = dnsServerEntry.address
            cell.isValidInput = viewModel.validateDNSDomainUserInput(dnsServerEntry.address)

            cell.onTextChange = { [weak self] cell in
                guard let self = self,
                      let indexPath = self.tableView?.indexPath(for: cell) else { return }

                if case let .dnsServer(entryIdentifier) = self.snapshot
                    .itemForIndexPath(indexPath)
                {
                    self.handleDNSEntryChange(entryIdentifier: entryIdentifier, cell: cell)
                }
            }

            cell.onReturnKey = { cell in
                cell.endEditing(false)
            }

            return cell
        }
    }

    private func setBlockAdvertising(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockAdvertising(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadCustomDNSFooter()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func setBlockTracking(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockTracking(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadCustomDNSFooter()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func setBlockMalware(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockMalware(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadCustomDNSFooter()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func setBlockAdultContent(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockAdultContent(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadCustomDNSFooter()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func setBlockGambling(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockGambling(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadCustomDNSFooter()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func setEnableCustomDNS(_ isEnabled: Bool) {
        viewModel.setEnableCustomDNS(isEnabled)

        reloadCustomDNSFooter()

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func handleDNSEntryChange(entryIdentifier: UUID, cell: SettingsDNSTextCell) {
        let string = cell.textField.text ?? ""
        let oldViewModel = viewModel

        viewModel.updateDNSEntry(entryIdentifier: entryIdentifier, newAddress: string)
        cell.isValidInput = viewModel.validateDNSDomainUserInput(string)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadCustomDNSFooter()
        }
    }

    private func addDNSServerEntry() {
        let oldViewModel = viewModel

        let newDNSEntry = DNSServerEntry(address: "")
        viewModel.customDNSDomains.append(newDNSEntry)

        let oldSnapshot = snapshot
        updateSnapshot()

        let diffResult = oldSnapshot.difference(snapshot)
        if let tableView = tableView {
            diffResult.apply(to: tableView, animateDifferences: true) { completed in
                if oldViewModel.customDNSPrecondition != self.viewModel.customDNSPrecondition {
                    self.reloadCustomDNSFooter()
                }

                if completed {
                    // Focus on the new entry text field.
                    let lastDNSEntry = self.snapshot.items(in: .customDNS).last { item in
                        if case let .dnsServer(entryIdentifier) = item {
                            return entryIdentifier == newDNSEntry.identifier
                        } else {
                            return false
                        }
                    }

                    if let lastDNSEntry = lastDNSEntry,
                       let indexPath = self.snapshot.indexPathForItem(lastDNSEntry)
                    {
                        let cell = self.tableView?.cellForRow(at: indexPath) as? SettingsDNSTextCell

                        self.tableView?.scrollToRow(at: indexPath, at: .bottom, animated: true)
                        cell?.textField.becomeFirstResponder()
                    }
                }
            }
        }
    }

    private func deleteDNSServerEntry(entryIdentifier: UUID) {
        let oldViewModel = viewModel
        let oldSnapshot = snapshot

        let entryIndex = viewModel.customDNSDomains.firstIndex { entry in
            return entry.identifier == entryIdentifier
        }

        guard let entryIndex = entryIndex else { return }

        viewModel.customDNSDomains.remove(at: entryIndex)
        updateSnapshot()

        let diffResult = oldSnapshot.difference(snapshot)

        if let tableView = tableView {
            diffResult.apply(to: tableView, animateDifferences: true) { completed in
                if oldViewModel.customDNSPrecondition != self.viewModel.customDNSPrecondition {
                    self.reloadCustomDNSFooter()
                }
            }
        }
    }

    private func reloadCustomDNSFooter() {
        let sectionIndex = snapshot.indexOfSection(.customDNS)!
        let indexPath = snapshot.indexPathForItem(.useCustomDNS)!

        // Reload footer view
        tableView?.performBatchUpdates {
            if let reusableView = tableView?
                .footerView(forSection: sectionIndex) as? SettingsStaticTextFooterView
            {
                configureFooterView(reusableView)
            }
        }

        // Reload "Use custom DNS" row
        if let cell = tableView?.cellForRow(at: indexPath) as? SettingsSwitchCell {
            cell.setEnabled(viewModel.customDNSPrecondition == .satisfied)
            cell.setOn(viewModel.effectiveEnableCustomDNS, animated: true)
        }
    }

    private func configureFooterView(_ reusableView: SettingsStaticTextFooterView) {
        let font = reusableView.titleLabel.font ?? UIFont.systemFont(ofSize: UIFont.systemFontSize)

        reusableView.titleLabel.attributedText = viewModel.customDNSPrecondition
            .attributedLocalizedDescription(isEditing: isEditing, preferredFont: font)
    }
}
