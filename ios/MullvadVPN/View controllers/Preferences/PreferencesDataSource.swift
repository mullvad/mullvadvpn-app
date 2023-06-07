//
//  PreferencesDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 05/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
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
        case dnsServerInfo
        case addDNSServer
        case wireGuardPort
        case wireGuardCustomPort
        #if DEBUG
        case wireGuardObfuscation
        case wireGuardObfuscationPort
        #endif
        var reusableViewClass: AnyClass {
            switch self {
            case .setting:
                return SettingsCell.self
            case .settingSwitch:
                return SettingsSwitchCell.self
            case .dnsServer:
                return SettingsDNSTextCell.self
            case .dnsServerInfo:
                return SettingsDNSInfoCell.self
            case .addDNSServer:
                return SettingsAddDNSEntryCell.self
            case .wireGuardPort:
                return SelectableSettingsCell.self
            case .wireGuardCustomPort:
                return SettingsInputCell.self
            #if DEBUG
            case .wireGuardObfuscation:
                return SelectableSettingsCell.self
            case .wireGuardObfuscationPort:
                return SelectableSettingsCell.self
            #endif
            }
        }
    }

    private enum HeaderFooterReuseIdentifiers: String, CaseIterable {
        case contentBlockerHeader
        case wireGuardPortHeader
        case spacer

        var reusableViewClass: AnyClass {
            switch self {
            case .contentBlockerHeader, .wireGuardPortHeader:
                return SettingsHeaderView.self
            case .spacer:
                return EmptyTableViewHeaderFooterView.self
            }
        }
    }

    enum InfoButtonItem {
        case contentBlockers
        case blockMalware
        case wireGuardPorts
        #if DEBUG
        case wireGuardObfuscation
        case wireGuardObfuscationPort
        #endif
    }

    enum Section: String, Hashable, CaseIterable {
        case contentBlockers
        case customDNS
        case wireGuardPorts
        #if DEBUG
        case wireGuardObfuscation
        case wireGuardObfuscationPort
        #endif
    }

    enum Item: Hashable {
        case blockAdvertising
        case blockTracking
        case blockMalware
        case blockAdultContent
        case blockGambling
        case blockSocialMedia
        case wireGuardPort(_ port: UInt16?)
        case wireGuardCustomPort
        case useCustomDNS
        case addDNSServer
        case dnsServer(_ uniqueID: UUID)
        case dnsServerInfo
        #if DEBUG
        case wireGuardObfuscationAutomatic
        case wireGuardObfuscationOn
        case wireGuardObfuscationOff
        case wireGuardObfuscationPort(_ port: UInt16)
        #endif

        static var contentBlockers: [Item] {
            [.blockAdvertising, .blockTracking, .blockMalware, .blockAdultContent, .blockGambling, .blockSocialMedia]
        }

        static var wireGuardPorts: [Item] {
            let defaultPorts = PreferencesViewModel.defaultWireGuardPorts.map {
                Item.wireGuardPort($0)
            }
            return [.wireGuardPort(nil)] + defaultPorts + [.wireGuardCustomPort]
        }

        #if DEBUG
        static var wireGuardObfuscation: [Item] {
            [.wireGuardObfuscationAutomatic, .wireGuardObfuscationOn, wireGuardObfuscationOff]
        }

        static var wireGuardObfuscationPort: [Item] {
            [.wireGuardObfuscationPort(0), wireGuardObfuscationPort(80), wireGuardObfuscationPort(5001)]
        }
        #endif
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
            case .blockSocialMedia:
                return "blockSocialMedias"
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
            case .dnsServerInfo:
                return "dnsServerInfo"
            #if DEBUG
            case .wireGuardObfuscationAutomatic:
                return "Automatic"
            case .wireGuardObfuscationOn:
                return "On (UDP-over-TCP)"
            case .wireGuardObfuscationOff:
                return "Off"
            case let .wireGuardObfuscationPort(port):
                if port == 0 {
                    return "Automatic"
                }
                return "\(port)"
            #endif
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
            case .dnsServerInfo:
                return .dnsServerInfo
            case .wireGuardPort:
                return .wireGuardPort
            case .wireGuardCustomPort:
                return .wireGuardCustomPort
            #if DEBUG
            case .wireGuardObfuscationAutomatic, .wireGuardObfuscationOn, .wireGuardObfuscationOff:
                return .wireGuardObfuscation
            case .wireGuardObfuscationPort:
                return .wireGuardObfuscationPort
            #endif
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

        super.init(tableView: tableView) { _, indexPath, itemIdentifier in
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

        updateSnapshot(animated: true)

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

    func update(from tunnelSettings: LatestTunnelSettings) {
        let newViewModel = PreferencesViewModel(from: tunnelSettings)
        let mergedViewModel = viewModel.merged(newViewModel)

        if viewModel != mergedViewModel {
            viewModel = mergedViewModel
        }

        updateSnapshot()
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
        #if DEBUG
        case .wireGuardPort, .wireGuardCustomPort, .wireGuardObfuscationAutomatic, .wireGuardObfuscationOn,
             .wireGuardObfuscationOff, .wireGuardObfuscationPort:
            return true
        #else
        case .wireGuardPort, .wireGuardCustomPort:
            return true
        #endif
        default:
            return false
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let item = itemIdentifier(for: indexPath)

        deselectAllRowsInSectionExceptRowAt(indexPath)

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

        #if DEBUG
        case .wireGuardObfuscationAutomatic:
            print("UDP over TCP Set to automatic")
        case .wireGuardObfuscationOn:
            print("turning on UDP over TCP")
        case .wireGuardObfuscationOff:
            print("Turning off UDP over TCP")
        case let .wireGuardObfuscationPort(port):
            print("Setting port to \(port)")
        #endif
        default:
            break
        }
    }

    func tableView(_ tableView: UITableView, willDeselectRowAt indexPath: IndexPath) -> IndexPath? {
        nil
    }

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
        case .wireGuardPorts:
            configureWireguardPortsHeader(view)
            return view

        #if DEBUG
        case .wireGuardObfuscation:
            configureObfuscationHeader(view)
            return view
        case .wireGuardObfuscationPort:
            configureObfuscationPortHeader(view)
            return view
        #endif
        default:
            return nil
        }
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        switch sectionIdentifier {
        case .wireGuardPorts:
            return tableView.dequeueReusableHeaderFooterView(
                withIdentifier: HeaderFooterReuseIdentifiers.spacer.rawValue
            )
        default:
            return nil
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
        #if DEBUG
        case .wireGuardObfuscationPort:
            return UIMetrics.TableView.sectionSpacing
        #else
        case .wireGuardPorts:
            return UIMetrics.TableView.sectionSpacing
        #endif

        default:
            return 0
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

    private func deselectAllRowsInSectionExceptRowAt(_ indexPath: IndexPath) {
        guard let indexPaths = tableView?.indexPathsForSelectedRows else { return }
        let rowsToDeselect = indexPaths.filter { $0.section == indexPath.section && $0.row != indexPath.row }

        rowsToDeselect.forEach {
            tableView?.deselectRow(at: $0, animated: false)
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

        if oldSnapshot.sectionIdentifiers.contains(.wireGuardPorts) {
            newSnapshot.appendItems(
                oldSnapshot.itemIdentifiers(inSection: .wireGuardPorts),
                toSection: .wireGuardPorts
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
        apply(snapshot, animatingDifferences: animated) { [weak self] in
            self?.selectRow(at: self?.indexPathForSelectedPort)
            completion?()
        }
    }

    private func reload(item: Item) {
        if let indexPath = indexPath(for: item),
           let cell = tableView?.cellForRow(at: indexPath) {
            preferencesCellFactory.configureCell(cell, item: item, indexPath: indexPath)
        }
    }

    private func setBlockAdvertising(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockAdvertising(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func setBlockTracking(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockTracking(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func setBlockMalware(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockMalware(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func setBlockAdultContent(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockAdultContent(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func setBlockGambling(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockGambling(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func setBlockSocialMedia(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setBlockSocialMedia(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    private func setEnableCustomDNS(_ isEnabled: Bool) {
        let oldViewModel = viewModel

        viewModel.setEnableCustomDNS(isEnabled)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadDnsServerInfo()
        }

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
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
            if let self {
                self.delegate?.preferencesDataSource(self, showInfo: .contentBlockers)
            }
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

    private func configureWireguardPortsHeader(_ header: SettingsHeaderView) {
        let title = NSLocalizedString(
            "WIRE_GUARD_PORTS_HEADER_LABEL",
            tableName: "Preferences",
            value: "WireGuard ports",
            comment: ""
        )

        header.titleLabel.text = title
        header.accessibilityCustomActionName = title
        header.infoButtonHandler = { [weak self] in
            if let self {
                self.delegate?.preferencesDataSource(self, showInfo: .wireGuardPorts)
            }
        }

        header.didCollapseHandler = { [weak self] headerView in
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

    #if DEBUG

    private func configureObfuscationHeader(_ header: SettingsHeaderView) {
        let title = NSLocalizedString(
            "OBFUSCATION_HEADER_LABEL",
            tableName: "Preferences",
            value: "WireGuard Obfuscation",
            comment: ""
        )

        header.titleLabel.text = title
        header.accessibilityCustomActionName = title
        header.didCollapseHandler = { [weak self] header in
            guard let self else { return }

            var snapshot = snapshot()
            if header.isExpanded {
                snapshot.deleteItems(Item.wireGuardObfuscation)
            } else {
                snapshot.appendItems(Item.wireGuardObfuscation, toSection: .wireGuardObfuscation)
            }
            header.isExpanded.toggle()
            applySnapshot(snapshot, animated: true)
        }

        header.infoButtonHandler = { [weak self] in
            self.map { $0.delegate?.preferencesDataSource($0, showInfo: .wireGuardObfuscation) }
        }
    }

    private func configureObfuscationPortHeader(_ header: SettingsHeaderView) {
        let title = NSLocalizedString(
            "OBFUSCATION_PORT_HEADER_LABEL",
            tableName: "Preferences",
            value: "UDP-over-TCP Port",
            comment: ""
        )

        header.titleLabel.text = title
        header.accessibilityCustomActionName = title
        header.didCollapseHandler = { [weak self] header in
            guard let self else { return }

            var snapshot = snapshot()
            if header.isExpanded {
                snapshot.deleteItems(Item.wireGuardObfuscationPort)
            } else {
                snapshot.appendItems(Item.wireGuardObfuscationPort, toSection: .wireGuardObfuscationPort)
            }
            header.isExpanded.toggle()
            applySnapshot(snapshot, animated: true)
        }

        header.infoButtonHandler = { [weak self] in
            self.map { $0.delegate?.preferencesDataSource($0, showInfo: .wireGuardObfuscationPort) }
        }
    }
    #endif
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

    func showInfo(for button: InfoButtonItem) {
        delegate?.preferencesDataSource(self, showInfo: button)
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

// swiftlint:disable:this file_length
