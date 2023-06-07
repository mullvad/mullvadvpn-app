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
    typealias InfoButtonHandler = (PreferencesDataSource.Item) -> Void

    enum CellReuseIdentifiers: String, CaseIterable {
        case setting
        case settingSwitch
        case dnsServer
        case addDNSServer
        case wireGuardPort
        case wireGuardCustomPort

        var reusableViewClass: AnyClass {
            switch self {
            case .setting:
                return SettingsCell.self
            case .settingSwitch:
                return SettingsSwitchCell.self
            case .dnsServer:
                return SettingsDNSTextCell.self
            case .addDNSServer:
                return SettingsAddDNSEntryCell.self
            case .wireGuardPort:
                return SelectableSettingsCell.self
            case .wireGuardCustomPort:
                return SettingsInputCell.self
            }
        }
    }

    private enum HeaderFooterReuseIdentifiers: String, CaseIterable {
        case contentBlockerHeader
        case wireGuardPortHeader
        case customDNSFooter
        case spacer

        var reusableViewClass: AnyClass {
            switch self {
            case .contentBlockerHeader, .wireGuardPortHeader:
                return SettingsHeaderView.self
            case .customDNSFooter:
                return SettingsStaticTextFooterView.self
            case .spacer:
                return EmptyTableViewHeaderFooterView.self
            }
        }
    }

    enum InfoButtonItem {
        case contentBlockers
        case blockMalware
        case wireGuardPorts
    }

    enum Section: String, Hashable, CaseIterable {
        case contentBlockers
        case customDNS
        case wireGuardPorts
    }

    enum Item: Hashable {
        case blockAdvertising
        case blockTracking
        case blockMalware
        case blockAdultContent
        case blockGambling
        case wireGuardPort(_ port: UInt16?)
        case wireGuardCustomPort
        case useCustomDNS
        case addDNSServer
        case dnsServer(_ uniqueID: UUID)

        static var contentBlockers: [Item] {
            return [.blockAdvertising, .blockTracking, .blockMalware, .blockAdultContent, .blockGambling]
        }

        static var wireGuardPorts: [Item] {
            let defaultPorts = PreferencesViewModel.defaultWireGuardPorts.map {
                Item.wireGuardPort($0)
            }
            return [.wireGuardPort(nil)] + defaultPorts + [.wireGuardCustomPort]
        }

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
            case let .wireGuardPort(port):
                if let port {
                    return "wireGuardPort(\(port))"
                } else {
                    return "wireGuardPort"
                }
            case .wireGuardCustomPort:
                return "wireGuardCustomPort"
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

        var reuseIdentifier: PreferencesDataSource.CellReuseIdentifiers {
            switch self {
            case .addDNSServer:
                return .addDNSServer
            case .dnsServer:
                return .dnsServer
            case .wireGuardPort:
                return .wireGuardPort
            case .wireGuardCustomPort:
                return .wireGuardCustomPort
            default:
                return .settingSwitch
            }
        }
    }

    private var isEditing = false

    private(set) var viewModel = PreferencesViewModel() { didSet {
        preferencesCellFactory.viewModel = viewModel
    }}
    private(set) var viewModelBeforeEditing = PreferencesViewModel()
    private let preferencesCellFactory: PreferencesCellFactory
    private weak var tableView: UITableView?

    weak var delegate: PreferencesDataSourceDelegate?

    var indexPathForSelectedPort: IndexPath? {
        let selectedItem: Item = viewModel.customWireGuardPort == nil
            ? .wireGuardPort(viewModel.wireGuardPort)
            : .wireGuardCustomPort

        return indexPath(for: selectedItem)
    }

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

    func setAvailablePortRanges(_ ranges: [[UInt16]]) {
        viewModel.availableWireGuardPortRanges = ranges
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

        viewModel.customDNSDomains.forEach { entry in
            self.reload(item: .dnsServer(entry.identifier))
        }

        if !editing, viewModelBeforeEditing != viewModel {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }

        selectRow(at: indexPathForSelectedPort)
    }

    func revertWireGuardPortCellToLastSelection() {
        guard let customPortCell = getCustomPortCell(), customPortCell.textField.isEditing else {
            return
        }

        customPortCell.textField.resignFirstResponder()

        if customPortCell.isValidInput {
            customPortCell.confirmInput()
        } else if let port = viewModel.customWireGuardPort {
            customPortCell.setInput(String(port))
            customPortCell.confirmInput()
        } else {
            customPortCell.reset()

            Item.wireGuardPorts.forEach { item in
                if case let .wireGuardPort(port) = item, port == viewModel.wireGuardPort {
                    selectRow(at: item)

                    return
                }
            }
        }
    }

    func update(from tunnelSettings: TunnelSettingsV2) {
        let newViewModel = PreferencesViewModel(from: tunnelSettings)
        let mergedViewModel = viewModel.merged(newViewModel)

        if viewModel != mergedViewModel {
            viewModel = mergedViewModel
        }

        updateSnapshot { [weak self] in
            self?.reloadCustomDNSFooter()
        }
    }

    // MARK: - UITableViewDataSource

    func tableView(
        _ tableView: UITableView,
        willDisplay cell: UITableViewCell,
        forRowAt indexPath: IndexPath
    ) {
        if indexPath == indexPathForSelectedPort {
            cell.setSelected(true, animated: false)
        }
    }

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
        switch itemIdentifier(for: indexPath) {
        case .wireGuardPort, .wireGuardCustomPort:
            return true
        default:
            return false
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let item = itemIdentifier(for: indexPath)

        switch item {
        case let .wireGuardPort(port):
            viewModel.setWireGuardPort(port)

            if let cell = getCustomPortCell() {
                cell.reset()
                cell.textField.resignFirstResponder()
            }

            delegate?.preferencesDataSource(self, didSelectPort: port)

        case .wireGuardCustomPort:
            getCustomPortCell()?.textField.becomeFirstResponder()

        default:
            break
        }
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        switch sectionIdentifier {
        case .contentBlockers:
            guard let view = tableView
                .dequeueReusableHeaderFooterView(
                    withIdentifier: HeaderFooterReuseIdentifiers.contentBlockerHeader.rawValue
                ) as? SettingsHeaderView else { return nil }
            configureContentBlockersHeader(view)
            return view

        case .wireGuardPorts:
            guard let view = tableView
                .dequeueReusableHeaderFooterView(
                    withIdentifier: HeaderFooterReuseIdentifiers.wireGuardPortHeader.rawValue
                ) as? SettingsHeaderView else { return nil }
            configureWireguardPortsHeader(view)
            return view

        default:
            return nil
        }
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        switch sectionIdentifier {
        case .contentBlockers:
            return nil

        case .customDNS:
            guard let view = tableView
                .dequeueReusableHeaderFooterView(
                    withIdentifier: HeaderFooterReuseIdentifiers.customDNSFooter.rawValue
                ) as? SettingsStaticTextFooterView else { return nil }
            configureFooterView(view)
            return view

        case .wireGuardPorts:
            return tableView.dequeueReusableHeaderFooterView(
                withIdentifier: HeaderFooterReuseIdentifiers.spacer.rawValue
            )
        }
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
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        switch sectionIdentifier {
        case .contentBlockers:
            return 0

        case .customDNS:
            switch viewModel.customDNSPrecondition {
            case .satisfied:
                return 0
            case .conflictsWithOtherSettings, .emptyDNSDomains:
                return UITableView.automaticDimension
            }

        case .wireGuardPorts:
            return UIMetrics.TableView.sectionSpacing
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

        if oldSnapshot.sectionIdentifiers.contains(.contentBlockers) {
            newSnapshot.appendItems(
                oldSnapshot.itemIdentifiers(inSection: .contentBlockers),
                toSection: .contentBlockers
            )
        }

        if oldSnapshot.sectionIdentifiers.contains(.wireGuardPorts) {
            newSnapshot.appendItems(
                oldSnapshot.itemIdentifiers(inSection: .wireGuardPorts),
                toSection: .wireGuardPorts
            )
        }

        newSnapshot.appendItems([.useCustomDNS], toSection: .customDNS)

        let dnsServerItems = viewModel.customDNSDomains.map { entry in
            return Item.dnsServer(entry.identifier)
        }
        newSnapshot.appendItems(dnsServerItems, toSection: .customDNS)

        if isEditing, viewModel.customDNSDomains.count < DNSSettings.maxAllowedCustomDNSDomains {
            newSnapshot.appendItems([.addDNSServer], toSection: .customDNS)
        }

        applySnapshot(newSnapshot, animated: animated, completion: completion)
    }

    private func applySnapshot(
        _ snapshot: NSDiffableDataSourceSnapshot<Section, Item>,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        apply(snapshot, animatingDifferences: animated) { [weak self] in
            self?.selectRow(at: self?.indexPathForSelectedPort)
            completion?()
        }
    }

    private func reload(item: Item) {
        if let indexPath = indexPath(for: item),
           let cell = tableView?.cellForRow(at: indexPath)
        {
            preferencesCellFactory.configureCell(cell, item: item, indexPath: indexPath)
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

    private func handleDNSEntryChange(with identifier: UUID, inputString: String) -> Bool {
        let oldViewModel = viewModel

        viewModel.updateDNSEntry(entryIdentifier: identifier, newAddress: inputString)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadCustomDNSFooter()
        }

        return viewModel.isDNSDomainUserInputValid(inputString)
    }

    private func addDNSServerEntry() {
        let oldViewModel = viewModel

        let newDNSEntry = DNSServerEntry(address: "")
        viewModel.customDNSDomains.append(newDNSEntry)

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

            if let lastDNSEntry,
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

        guard let entryIndex else { return }

        viewModel.customDNSDomains.remove(at: entryIndex)

        updateSnapshot(animated: true) { [weak self] in
            if oldViewModel.customDNSPrecondition != self?.viewModel.customDNSPrecondition {
                self?.reloadCustomDNSFooter()
            }
        }
    }

    private func reloadCustomDNSFooter() {
        reload(item: .useCustomDNS)

        let sectionIndex = snapshot().indexOfSection(.customDNS)!

        UIView.performWithoutAnimation {
            if let reusableView = tableView?.footerView(forSection: sectionIndex) as? SettingsStaticTextFooterView {
                configureFooterView(reusableView)
            }
        }
    }

    private func configureContentBlockersHeader(_ reusableView: SettingsHeaderView) {
        reusableView.titleLabel.text = NSLocalizedString(
            "CONTENT_BLOCKERS_HEADER_LABEL",
            tableName: "Preferences",
            value: "DNS content blockers",
            comment: ""
        )

        reusableView.infoButtonHandler = { [weak self] in
            if let self {
                self.delegate?.preferencesDataSource(self, showInfo: .contentBlockers)
            }
        }

        reusableView.didCollapseHandler = { [weak self] headerView in
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

    private func configureWireguardPortsHeader(_ reusableView: SettingsHeaderView) {
        reusableView.titleLabel.text = NSLocalizedString(
            "WIRE_GUARD_PORTS_HEADER_LABEL",
            tableName: "Preferences",
            value: "WireGuard ports",
            comment: ""
        )

        reusableView.infoButtonHandler = { [weak self] in
            if let self {
                self.delegate?.preferencesDataSource(self, showInfo: .wireGuardPorts)
            }
        }

        reusableView.didCollapseHandler = { [weak self] headerView in
            guard let self else { return }

            var snapshot = self.snapshot()
            var updateTimeDelay = 0.0

            if headerView.isExpanded {
                if let customPortCell = getCustomPortCell(), customPortCell.textField.isEditing {
                    revertWireGuardPortCellToLastSelection()
                    updateTimeDelay = 0.5
                }

                snapshot.deleteItems(Item.wireGuardPorts)
            } else {
                snapshot.appendItems(Item.wireGuardPorts, toSection: .wireGuardPorts)
            }

            // The update should be delayed when we're reverting an ongoing change, to give the
            // user just enough time to notice it.
            DispatchQueue.main.asyncAfter(deadline: .now() + updateTimeDelay) { [weak self] in
                headerView.isExpanded.toggle()

                self?.applySnapshot(snapshot, animated: true)
            }
        }
    }

    private func configureFooterView(_ reusableView: SettingsStaticTextFooterView) {
        let font = reusableView.titleLabel.font ?? UIFont.systemFont(ofSize: UIFont.systemFontSize)

        reusableView.titleLabel.attributedText = viewModel.customDNSPrecondition
            .attributedLocalizedDescription(isEditing: isEditing, preferredFont: font)

        reusableView.titleLabel.sizeToFit()

        // Applying background color of table view hides overflow from contracting cells below.
        reusableView.contentView.backgroundColor = tableView?.backgroundColor
    }

    private func selectRow(at indexPath: IndexPath?, animated: Bool = false) {
        tableView?.selectRow(at: indexPath, animated: false, scrollPosition: .none)
    }

    private func selectRow(at item: Item?, animated: Bool = false) {
        guard let item else { return }

        tableView?.selectRow(at: indexPath(for: item), animated: false, scrollPosition: .none)
    }

    private func getCustomPortCell() -> SettingsInputCell? {
        if let customPortIndexPath = indexPath(for: .wireGuardCustomPort) {
            return tableView?.cellForRow(at: customPortIndexPath) as? SettingsInputCell
        }

        return nil
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

    func showInfo(for item: InfoButtonItem) {
        delegate?.preferencesDataSource(self, showInfo: item)
    }

    func addCustomPort(_ port: UInt16) {
        viewModel.setWireGuardPort(port)
        delegate?.preferencesDataSource(self, didSelectPort: port)
    }

    func selectCustomPortEntry(_ port: UInt16) -> Bool {
        if indexPathForSelectedPort != indexPath(for: .wireGuardCustomPort) {
            selectRow(at: .wireGuardCustomPort)
        }

        return viewModel.isPortWithinValidWireGuardRanges(port)
    }
}
