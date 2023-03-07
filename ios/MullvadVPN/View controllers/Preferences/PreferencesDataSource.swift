//
//  PreferencesDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 05/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

final class PreferencesDataSource: UITableViewDiffableDataSource<
    PreferencesDataSource.Section,
    PreferencesDataSource.Item
>, UITableViewDelegate {
    enum CellReuseIdentifiers: String, CaseIterable {
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

    enum Section: String, Hashable {
        case mullvadDNS
        case customDNS
    }

    enum Item: Hashable {
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

    private(set) var viewModel = PreferencesViewModel()
    private(set) var viewModelBeforeEditing = PreferencesViewModel()
    private let preferencesCellFactory: PreferencesCellFactory
    private weak var tableView: UITableView?

    weak var delegate: PreferencesDataSourceDelegate?

    init(tableView: UITableView) {
        self.tableView = tableView

        let preferencesCellFactory = PreferencesCellFactory(
            tableView: tableView,
            viewModel: viewModel
        )
        self.preferencesCellFactory = preferencesCellFactory

        super.init(tableView: tableView) { tableView, indexPath, itemIdentifier in
            preferencesCellFactory.makeCell(for: itemIdentifier, indexPath: indexPath)
        }

        tableView.delegate = self
        preferencesCellFactory.delegate = self

        registerClasses()
    }

    func setEditing(_ editing: Bool, animated: Bool) {
        guard isEditing != editing else { return }

        isEditing = editing
        preferencesCellFactory.isEditing = isEditing

        if editing {
            viewModelBeforeEditing = viewModel
        } else {
            viewModel.sanitizeCustomDNSEntries()
        }

        updateSnapshot()
        reloadCustomDNSFooter()

        updateCellFactory(with: viewModel)
        viewModel.customDNSDomains.forEach { entry in
            self.reload(item: .dnsServer(entry.identifier))
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
            reloadCustomDNSFooter()
        }
    }

    // MARK: - UITableViewDataSource

    override func tableView(_ tableView: UITableView, canEditRowAt indexPath: IndexPath) -> Bool {
        // Disable swipe to delete when not editing the table view
        guard isEditing else { return false }

        let item = itemIdentifier(for: indexPath)

        switch item {
        case .dnsServer, .addDNSServer:
            return true
        default:
            return false
        }
    }

    override func tableView(
        _ tableView: UITableView,
        commit editingStyle: UITableViewCell.EditingStyle,
        forRowAt indexPath: IndexPath
    ) {
        let item = itemIdentifier(for: indexPath)

        if case .addDNSServer = item, editingStyle == .insert {
            addDNSServerEntry()
        }

        if case let .dnsServer(entryIdentifier) = item, editingStyle == .delete {
            deleteDNSServerEntry(entryIdentifier: entryIdentifier)
        }
    }

    override func tableView(_ tableView: UITableView, canMoveRowAt indexPath: IndexPath) -> Bool {
        let item = itemIdentifier(for: indexPath)

        switch item {
        case .dnsServer:
            return true
        default:
            return false
        }
    }

    override func tableView(
        _ tableView: UITableView,
        moveRowAt sourceIndexPath: IndexPath,
        to destinationIndexPath: IndexPath
    ) {
        let sourceItem = itemIdentifier(for: sourceIndexPath)!
        let destinationItem = itemIdentifier(for: destinationIndexPath)!

        guard case let .dnsServer(sourceIdentifier) = sourceItem,
              case let .dnsServer(targetIdentifier) = destinationItem,
              let sourceIndex = viewModel.indexOfDNSEntry(entryIdentifier: sourceIdentifier),
              let destinationIndex = viewModel.indexOfDNSEntry(entryIdentifier: targetIdentifier)
        else { return }

        let removedEntry = viewModel.customDNSDomains.remove(at: sourceIndex)
        viewModel.customDNSDomains.insert(removedEntry, at: destinationIndex)

        updateCellFactory(with: viewModel)
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
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

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
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

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
        let item = itemIdentifier(for: indexPath)

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
        let sectionIdentifier = snapshot().sectionIdentifiers[sourceIndexPath.section]
        guard case .customDNS = sectionIdentifier else { return sourceIndexPath }

        let items = snapshot().itemIdentifiers(inSection: sectionIdentifier)

        let indexPathForFirstRow = items.first(where: Item.isDNSServerItem).flatMap { item in
            return indexPath(for: item)
        }

        let indexPathForLastRow = items.last(where: Item.isDNSServerItem).flatMap { item in
            return indexPath(for: item)
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

    private func updateSnapshot(animated: Bool = false, completion: (() -> Void)? = nil) {
        var snapshot = NSDiffableDataSourceSnapshot<Section, Item>()

        snapshot.appendSections([.mullvadDNS, .customDNS])
        snapshot.appendItems(
            [.blockAdvertising, .blockTracking, .blockMalware, .blockAdultContent, .blockGambling],
            toSection: .mullvadDNS
        )
        snapshot.appendItems([.useCustomDNS], toSection: .customDNS)

        let dnsServerItems = viewModel.customDNSDomains.map { entry in
            return Item.dnsServer(entry.identifier)
        }
        snapshot.appendItems(dnsServerItems, toSection: .customDNS)

        if isEditing, viewModel.customDNSDomains.count < DNSSettings.maxAllowedCustomDNSDomains {
            snapshot.appendItems([.addDNSServer], toSection: .customDNS)
        }

        apply(snapshot, completion: completion)
    }

    private func reload(item: Item) {
        if let indexPath = indexPath(for: item),
           let cell = tableView?.cellForRow(at: indexPath)
        {
            preferencesCellFactory.configureCell(cell, item: item, indexPath: indexPath)
        }
    }

    func updateCellFactory(with viewModel: PreferencesViewModel) {
        preferencesCellFactory.viewModel = viewModel
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

    private func handleDNSEntryChange(with identifier: UUID, inputString: String) -> Bool {
        let oldViewModel = viewModel

        viewModel.updateDNSEntry(entryIdentifier: identifier, newAddress: inputString)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadCustomDNSFooter()
        }

        return viewModel.validateDNSDomainUserInput(inputString)
    }

    private func addDNSServerEntry() {
        let oldViewModel = viewModel

        let newDNSEntry = DNSServerEntry(address: "")
        viewModel.customDNSDomains.append(newDNSEntry)

        updateCellFactory(with: viewModel)
        updateSnapshot(animated: true) { [weak self] in
            if oldViewModel.customDNSPrecondition != self?.viewModel.customDNSPrecondition {
                self?.reloadCustomDNSFooter()
            }

            // Focus on the new entry text field.
            let lastDNSEntry = self?.snapshot().itemIdentifiers(inSection: .customDNS)
                .last { item in
                    if case let .dnsServer(entryIdentifier) = item {
                        return entryIdentifier == newDNSEntry.identifier
                    } else {
                        return false
                    }
                }

            if let lastDNSEntry = lastDNSEntry,
               let indexPath = self?.indexPath(for: lastDNSEntry)
            {
                let cell = self?.tableView?.cellForRow(at: indexPath) as? SettingsDNSTextCell

                self?.tableView?.scrollToRow(at: indexPath, at: .bottom, animated: true)
                cell?.textField.becomeFirstResponder()
            }
        }
    }

    private func deleteDNSServerEntry(entryIdentifier: UUID) {
        let oldViewModel = viewModel

        let entryIndex = viewModel.customDNSDomains.firstIndex { entry in
            return entry.identifier == entryIdentifier
        }

        guard let entryIndex = entryIndex else { return }

        viewModel.customDNSDomains.remove(at: entryIndex)

        updateCellFactory(with: viewModel)
        updateSnapshot(animated: true) { [weak self] in
            if oldViewModel.customDNSPrecondition != self?.viewModel.customDNSPrecondition {
                self?.reloadCustomDNSFooter()
            }
        }
    }

    private func reloadCustomDNSFooter() {
        updateCellFactory(with: viewModel)
        reload(item: .useCustomDNS)

        let sectionIndex = snapshot().indexOfSection(.customDNS)!
        if let reusableView = tableView?
            .footerView(forSection: sectionIndex) as? SettingsStaticTextFooterView
        {
            configureFooterView(reusableView)
        }
    }

    private func configureFooterView(_ reusableView: SettingsStaticTextFooterView) {
        let font = reusableView.titleLabel.font ?? UIFont.systemFont(ofSize: UIFont.systemFontSize)

        reusableView.titleLabel.attributedText = viewModel.customDNSPrecondition
            .attributedLocalizedDescription(isEditing: isEditing, preferredFont: font)
    }
}

extension PreferencesDataSource: PreferencesCellEventHandler {
    func didChangeState(for item: Item, isOn: Bool) {
        switch item {
        case .blockAdvertising:
            setBlockAdvertising(isOn)

        case .blockTracking:
            setBlockTracking(isOn)

        case .blockMalware:
            setBlockMalware(isOn)

        case .blockAdultContent:
            setBlockAdultContent(isOn)

        case .blockGambling:
            setBlockGambling(isOn)

        case .useCustomDNS:
            setEnableCustomDNS(isOn)

        default:
            break
        }
    }

    func addDNSEntry() {
        addDNSServerEntry()
    }

    func didChangeDNSEntry(
        with identifier: UUID,
        inputString: String
    ) -> Bool {
        return handleDNSEntryChange(with: identifier, inputString: inputString)
    }
}
