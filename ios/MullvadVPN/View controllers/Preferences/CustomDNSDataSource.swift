//
//  CustomDNSDataSource.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-11-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

final class CustomDNSDataSource: UITableViewDiffableDataSource<
    CustomDNSDataSource.Section,
    CustomDNSDataSource.Item
>, UITableViewDelegate {
    typealias InfoButtonHandler = (Item) -> Void

    enum CellReuseIdentifiers: String, CaseIterable {
        case settingSwitch
        case dnsServer
        case dnsServerInfo
        case addDNSServer

        var reusableViewClass: AnyClass {
            switch self {
            case .settingSwitch:
                return SettingsSwitchCell.self
            case .dnsServer:
                return SettingsDNSTextCell.self
            case .dnsServerInfo:
                return SettingsDNSInfoCell.self
            case .addDNSServer:
                return SettingsAddDNSEntryCell.self
            }
        }
    }

    private enum HeaderFooterReuseIdentifiers: String, CaseIterable {
        case contentBlockerHeader

        var reusableViewClass: AnyClass {
            return SettingsHeaderView.self
        }
    }

    enum Section: String, Hashable, CaseIterable {
        case contentBlockers
        case customDNS
    }

    enum Item: Hashable {
        case blockAdvertising
        case blockTracking
        case blockMalware
        case blockAdultContent
        case blockGambling
        case blockSocialMedia
        case useCustomDNS
        case addDNSServer
        case dnsServer(_ uniqueID: UUID)
        case dnsServerInfo

        static var contentBlockers: [Item] {
            [.blockAdvertising, .blockTracking, .blockMalware, .blockAdultContent, .blockGambling, .blockSocialMedia]
        }

        var accessibilityIdentifier: AccessibilityIdentifier {
            switch self {
            case .blockAdvertising:
                return .blockAdvertising
            case .blockTracking:
                return .blockTracking
            case .blockMalware:
                return .blockMalware
            case .blockGambling:
                return .blockGambling
            case .blockAdultContent:
                return .blockAdultContent
            case .blockSocialMedia:
                return .blockSocialMedia
            case .useCustomDNS:
                return .useCustomDNS
            case .addDNSServer:
                return .addDNSServer
            case .dnsServer:
                return .dnsServer
            case .dnsServerInfo:
                return .dnsServerInfo
            }
        }

        static func isDNSServerItem(_ item: Item) -> Bool {
            if case .dnsServer = item {
                return true
            } else {
                return false
            }
        }

        var reuseIdentifier: CellReuseIdentifiers {
            switch self {
            case .addDNSServer:
                return .addDNSServer
            case .dnsServer:
                return .dnsServer
            case .dnsServerInfo:
                return .dnsServerInfo
            default:
                return .settingSwitch
            }
        }
    }

    private var isEditing = false

    private(set) var viewModel = PreferencesViewModel() { didSet {
        cellFactory.viewModel = viewModel
    }}
    private(set) var viewModelBeforeEditing = PreferencesViewModel()
    private let cellFactory: CustomDNSCellFactory
    private weak var tableView: UITableView?

    weak var delegate: PreferencesDataSourceDelegate?

    init(tableView: UITableView) {
        self.tableView = tableView

        let cellFactory = CustomDNSCellFactory(
            tableView: tableView,
            viewModel: viewModel
        )
        self.cellFactory = cellFactory

        super.init(tableView: tableView) { _, indexPath, item in
            cellFactory.makeCell(for: item, indexPath: indexPath)
        }

        tableView.delegate = self
        cellFactory.delegate = self

        registerClasses()
    }

    func setAvailablePortRanges(_ ranges: [[UInt16]]) {
        viewModel.availableWireGuardPortRanges = ranges
    }

    func setEditing(_ editing: Bool, animated: Bool) {
        guard isEditing != editing else { return }

        isEditing = editing
        cellFactory.isEditing = isEditing

        if editing {
            viewModelBeforeEditing = viewModel
        } else {
            viewModel.sanitizeCustomDNSEntries()
        }

        updateSnapshot(animated: true)

        viewModel.customDNSDomains.forEach { entry in
            reload(item: .dnsServer(entry.identifier))
        }

        if !editing, viewModelBeforeEditing != viewModel {
            delegate?.didChangeViewModel(viewModel)
        }
    }

    func update(from tunnelSettings: LatestTunnelSettings) {
        let newViewModel = PreferencesViewModel(from: tunnelSettings)
        let mergedViewModel = viewModel.merged(newViewModel)

        if viewModel != mergedViewModel {
            viewModel = mergedViewModel
        }

        updateSnapshot()
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

        updateSnapshot()
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        false
    }

    // Disallow selection for tapping on a selected setting
    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        guard let view = tableView
            .dequeueReusableHeaderFooterView(
                withIdentifier: HeaderFooterReuseIdentifiers.contentBlockerHeader
                    .rawValue
            ) as? SettingsHeaderView else { return nil }

        switch sectionIdentifier {
        case .contentBlockers:
            configureContentBlockersHeader(view)
            return view
        default:
            return nil
        }
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        nil
    }

    func tableView(_ tableView: UITableView, heightForHeaderInSection section: Int) -> CGFloat {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        switch sectionIdentifier {
        case .customDNS:
            return 0

        default:
            return UITableView.automaticDimension
        }
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        0
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
            indexPath(for: item)
        }

        let indexPathForLastRow = items.last(where: Item.isDNSServerItem).flatMap { item in
            indexPath(for: item)
        }

        guard let indexPathForFirstRow,
              let indexPathForLastRow else { return sourceIndexPath }

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
        var newSnapshot = NSDiffableDataSourceSnapshot<Section, Item>()
        let oldSnapshot = snapshot()

        newSnapshot.appendSections(Section.allCases)

        // Append sections

        if oldSnapshot.sectionIdentifiers.contains(.contentBlockers) {
            newSnapshot.appendItems(
                oldSnapshot.itemIdentifiers(inSection: .contentBlockers),
                toSection: .contentBlockers
            )
        }

        // Append DNS settings

        newSnapshot.appendItems([.useCustomDNS], toSection: .customDNS)

        let dnsServerItems = viewModel.customDNSDomains.map { entry in
            Item.dnsServer(entry.identifier)
        }
        newSnapshot.appendItems(dnsServerItems, toSection: .customDNS)

        if isEditing, viewModel.customDNSDomains.count < DNSSettings.maxAllowedCustomDNSDomains {
            newSnapshot.appendItems([.addDNSServer], toSection: .customDNS)
        }

        // Append/update DNS server info.

        if viewModel.customDNSPrecondition == .satisfied {
            newSnapshot.deleteItems([.dnsServerInfo])
        } else {
            if newSnapshot.itemIdentifiers(inSection: .customDNS).contains(.dnsServerInfo) {
                newSnapshot.reloadItems([.dnsServerInfo])
            } else {
                newSnapshot.appendItems([.dnsServerInfo], toSection: .customDNS)
            }
        }

        applySnapshot(newSnapshot, animated: animated, completion: completion)
    }

    private func applySnapshot(
        _ snapshot: NSDiffableDataSourceSnapshot<Section, Item>,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        apply(snapshot, animatingDifferences: animated) {
            completion?()
        }
    }

    private func reload(item: Item) {
        if let indexPath = indexPath(for: item),
           let cell = tableView?.cellForRow(at: indexPath) {
            cellFactory.configureCell(cell, item: item, indexPath: indexPath)
        }
    }

    private func setBlockAdvertising(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockAdvertising(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.didChangeViewModel(viewModel)
        }
    }

    private func setBlockTracking(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockTracking(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.didChangeViewModel(viewModel)
        }
    }

    private func setBlockMalware(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockMalware(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.didChangeViewModel(viewModel)
        }
    }

    private func setBlockAdultContent(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockAdultContent(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.didChangeViewModel(viewModel)
        }
    }

    private func setBlockGambling(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockGambling(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.didChangeViewModel(viewModel)
        }
    }

    private func setBlockSocialMedia(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockSocialMedia(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.didChangeViewModel(viewModel)
        }
    }

    private func setEnableCustomDNS(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setEnableCustomDNS(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.didChangeViewModel(viewModel)
        }
    }

    private func handleDNSEntryChange(with identifier: UUID, inputString: String) -> Bool {
        let oldViewModel = viewModel

        viewModel.updateDNSEntry(entryIdentifier: identifier, newAddress: inputString)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        return viewModel.isDNSDomainUserInputValid(inputString)
    }

    private func addDNSServerEntry() {
        let newDNSEntry = DNSServerEntry(address: "")
        viewModel.customDNSDomains.append(newDNSEntry)

        updateSnapshot(animated: true) { [weak self] in
            // Focus on the new entry text field.
            let lastDNSEntry = self?.snapshot().itemIdentifiers(inSection: .customDNS)
                .last { item in
                    if case let .dnsServer(entryIdentifier) = item {
                        return entryIdentifier == newDNSEntry.identifier
                    } else {
                        return false
                    }
                }

            if let lastDNSEntry,
               let indexPath = self?.indexPath(for: lastDNSEntry) {
                let cell = self?.tableView?.cellForRow(at: indexPath) as? SettingsDNSTextCell

                self?.tableView?.scrollToRow(at: indexPath, at: .bottom, animated: true)
                cell?.textField.becomeFirstResponder()
            }
        }
    }

    private func deleteDNSServerEntry(entryIdentifier: UUID) {
        let entryIndex = viewModel.customDNSDomains.firstIndex { entry in
            entry.identifier == entryIdentifier
        }

        guard let entryIndex else { return }

        viewModel.customDNSDomains.remove(at: entryIndex)

        reload(item: .useCustomDNS)
        updateSnapshot(animated: true)
    }

    private func reloadDnsServerInfo() {
        var snapshot = snapshot()

        reload(item: .useCustomDNS)

        if viewModel.customDNSPrecondition == .satisfied {
            snapshot.deleteItems([.dnsServerInfo])
        } else {
            if snapshot.itemIdentifiers(inSection: .customDNS).contains(.dnsServerInfo) {
                snapshot.reloadItems([.dnsServerInfo])
            } else {
                snapshot.appendItems([.dnsServerInfo], toSection: .customDNS)
            }
        }

        apply(snapshot, animatingDifferences: true)
    }

    private func configureContentBlockersHeader(_ header: SettingsHeaderView) {
        let title = NSLocalizedString(
            "CONTENT_BLOCKERS_HEADER_LABEL",
            tableName: "Preferences",
            value: "DNS content blockers",
            comment: ""
        )

        header.titleLabel.text = title
        header.accessibilityCustomActionName = title

        header.infoButtonHandler = { [weak self] in
            self?.delegate?.showInfo(for: .contentBlockers)
        }

        header.didCollapseHandler = { [weak self] headerView in
            guard let self else { return }

            var snapshot = self.snapshot()

            if headerView.isExpanded {
                snapshot.deleteItems(Item.contentBlockers)
            } else {
                snapshot.appendItems(Item.contentBlockers, toSection: .contentBlockers)
            }

            headerView.isExpanded.toggle()
            self.apply(snapshot, animatingDifferences: true)
        }
    }
}

extension CustomDNSDataSource: CustomDNSCellEventHandler {
    func didChangeState(for preference: Item, isOn: Bool) {
        switch preference {
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

        case .blockSocialMedia:
            setBlockSocialMedia(isOn)

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
        handleDNSEntryChange(with: identifier, inputString: inputString)
    }

    func showInfo(for button: PreferencesInfoButtonItem) {
        delegate?.showInfo(for: button)
    }
}

// swiftlint:disable:this file_length
