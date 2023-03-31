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
>, UITableViewDelegate, PreferencesCellEventHandler {
    enum CellReuseIdentifiers: String, CaseIterable {
        case settingSwitch
        case dnsServer
        case addDNSServer
        case addConnectedNetwork
        case addTrustedNetwork
        case trustedNetwork

        var reusableViewClass: AnyClass {
            switch self {
            case .settingSwitch:
                return SettingsSwitchCell.self
            case .dnsServer:
                return SettingsDNSTextCell.self
            case .addDNSServer, .addTrustedNetwork:
                return SettingsAddEntryCell.self
            case .addConnectedNetwork:
                return AddConnectedNetworkCell.self
            case .trustedNetwork:
                return TrustedNetworkTextCell.self
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
        case trustedNetworks
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

        case useTrustedNetworks
        case addConnectedNetwork
        case addTrustedNetwork
        case trustedNetwork(_ uniqueID: UUID)

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
            case .useTrustedNetworks:
                return "useTrustedNetworks"
            case .addConnectedNetwork:
                return "addConnectedNetwork"
            case .addTrustedNetwork:
                return "addTrustedNetwork"
            case let .trustedNetwork(uuid):
                return "trustedNetwork(\(uuid.uuidString))"
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
            viewModel.sanitizeData()
        }

        updateSnapshot(animated: true)
        reloadCustomDNSFooter()

        updateCellFactory(with: viewModel)
        viewModel.customDNSDomains.forEach { entry in
            self.reload(item: .dnsServer(entry.identifier))
        }

        if !editing, viewModelBeforeEditing != viewModel {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    func update(from tunnelSettings: TunnelSettingsV2) {
        let newViewModel = PreferencesViewModel(from: tunnelSettings)
        let mergedViewModel = viewModel.merged(newViewModel)

        if viewModel != mergedViewModel {
            viewModel = mergedViewModel
        }

        updateCellFactory(with: viewModel)
        updateSnapshot()
        reloadCustomDNSFooter()
    }

    func setConnectedWifiNetwork(_ network: ConnectedWifiNetwork?) {
        var newViewModel = viewModel

        newViewModel.connectedNetwork = network

        if viewModel != newViewModel {
            viewModel = newViewModel

            updateCellFactory(with: newViewModel)
            updateSnapshot(animated: true)
        }
    }

    // MARK: - UITableViewDataSource

    override func tableView(_ tableView: UITableView, canEditRowAt indexPath: IndexPath) -> Bool {
        // Disable swipe to delete when not editing the table view
        guard isEditing else { return false }

        let item = itemIdentifier(for: indexPath)

        switch item {
        case .dnsServer, .addDNSServer, .addConnectedNetwork, .addTrustedNetwork, .trustedNetwork:
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
            addDNSEntry()
        }

        if case let .dnsServer(entryIdentifier) = item, editingStyle == .delete {
            deleteDNSServerEntry(entryIdentifier: entryIdentifier)
        }

        if case .addTrustedNetwork = item {
            addTrustedNetworkEntry(ssid: nil, beginEditing: true)
        }

        if case .addConnectedNetwork = item, let ssid = viewModel.connectedNetwork?.ssid, editingStyle == .insert {
            addTrustedNetworkEntry(ssid: ssid, beginEditing: false)
        }

        if case let .trustedNetwork(entryIdentifier) = item, editingStyle == .delete {
            deleteTrustedNetworkEntry(entryIdentifier: entryIdentifier)
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
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        switch sectionIdentifier {
        case .mullvadDNS, .customDNS:
            return tableView.dequeueReusableHeaderFooterView(
                withIdentifier: HeaderFooterReuseIdentifiers.spacer.rawValue
            )

        case .trustedNetworks:
            return nil
        }
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        switch sectionIdentifier {
        case .mullvadDNS, .trustedNetworks:
            return nil

        case .customDNS:
            guard let reusableView = tableView.dequeueReusableHeaderFooterView(
                withIdentifier: HeaderFooterReuseIdentifiers.customDNSFooter.rawValue
            ) as? SettingsStaticTextFooterView else {
                return nil
            }

            configureFooterView(reusableView)

            return reusableView
        }
    }

    func tableView(_ tableView: UITableView, heightForHeaderInSection section: Int) -> CGFloat {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        switch sectionIdentifier {
        case .mullvadDNS, .customDNS:
            return UIMetrics.sectionSpacing
        case .trustedNetworks:
            return UITableView.automaticDimension
        }
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        switch sectionIdentifier {
        case .mullvadDNS, .trustedNetworks:
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

    func tableView(_ tableView: UITableView, editingStyleForRowAt indexPath: IndexPath) -> UITableViewCell.EditingStyle
    {
        let item = itemIdentifier(for: indexPath)

        switch item {
        case .dnsServer, .trustedNetwork:
            return .delete
        case .addDNSServer, .addConnectedNetwork, .addTrustedNetwork:
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

        snapshot.appendSections([.mullvadDNS, .customDNS, .trustedNetworks])
        snapshot.appendItems(
            [.blockAdvertising, .blockTracking, .blockMalware, .blockAdultContent, .blockGambling],
            toSection: .mullvadDNS
        )
        snapshot.appendItems([.useCustomDNS], toSection: .customDNS)

        let dnsServerItems = viewModel.customDNSDomains.map { Item.dnsServer($0.identifier) }
        snapshot.appendItems(dnsServerItems, toSection: .customDNS)

        if isEditing, viewModel.customDNSDomains.count < DNSSettings.maxAllowedCustomDNSDomains {
            snapshot.appendItems([.addDNSServer], toSection: .customDNS)
        }

        snapshot.appendItems([.useTrustedNetworks], toSection: .trustedNetworks)

        if !viewModel.trustedNetworks.isEmpty {
            let trustedNetworkItems = viewModel.trustedNetworks.map { Item.trustedNetwork($0.identifier) }
            snapshot.appendItems(trustedNetworkItems, toSection: .trustedNetworks)
        }

        if isEditing {
            snapshot.appendItems([.addTrustedNetwork], toSection: .trustedNetworks)
        }

        if let connectedNetwork = viewModel.connectedNetwork,
           !viewModel.containsTrustedNetwork(ssid: connectedNetwork.ssid), isEditing
        {
            snapshot.appendItems([.addConnectedNetwork], toSection: .trustedNetworks)
        }

        apply(snapshot, animatingDifferences: animated, completion: completion)
    }

    private func reload(item: Item) {
        if let indexPath = indexPath(for: item), let cell = tableView?.cellForRow(at: indexPath) {
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

    func didChangeDNSEntry(with identifier: UUID, inputString: String) {
        let oldViewModel = viewModel

        viewModel.updateDNSEntry(entryIdentifier: identifier, newAddress: inputString)

        if oldViewModel.customDNSPrecondition != viewModel.customDNSPrecondition {
            reloadCustomDNSFooter()
        }
    }

    func addDNSEntry() {
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
        if let reusableView = tableView?.footerView(forSection: sectionIndex) as? SettingsStaticTextFooterView {
            configureFooterView(reusableView)
        }
    }

    private func configureFooterView(_ reusableView: SettingsStaticTextFooterView) {
        let font = reusableView.titleLabel.font ?? UIFont.systemFont(ofSize: UIFont.systemFontSize)

        reusableView.titleLabel.attributedText = viewModel.customDNSPrecondition
            .attributedLocalizedDescription(isEditing: isEditing, preferredFont: font)
    }

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

        case .useTrustedNetworks:
            setEnableTrustedNetworks(isOn)

        default:
            break
        }
    }

    private func setEnableTrustedNetworks(_ isEnabled: Bool) {
        viewModel.useTrustedNetworks = isEnabled

        if !isEditing {
            delegate?.preferencesDataSource(self, didChangeViewModel: viewModel)
        }
    }

    func addTrustedNetworkEntry(ssid: String?, beginEditing: Bool) {
        guard let newTrustedNetworkEntry = viewModel.addTrustedNetwork(ssid) else { return }

        updateCellFactory(with: viewModel)
        updateSnapshot(animated: true) { [weak self] in
            guard let self = self, beginEditing else { return }

            // Focus on the new entry text field.
            let lastTrustedNetworkEntry = self.snapshot().itemIdentifiers(inSection: .trustedNetworks)
                .last { item in
                    if case let .trustedNetwork(entryIdentifier) = item {
                        return entryIdentifier == newTrustedNetworkEntry.identifier
                    } else {
                        return false
                    }
                }

            if let lastTrustedNetworkEntry = lastTrustedNetworkEntry,
               let indexPath = self.indexPath(for: lastTrustedNetworkEntry)
            {
                let cell = self.tableView?.cellForRow(at: indexPath) as? TrustedNetworkTextCell

                self.tableView?.scrollToRow(at: indexPath, at: .bottom, animated: true)
                cell?.textField.becomeFirstResponder()
            }
        }

        reload(item: .useTrustedNetworks)
    }

    func deleteTrustedNetworkEntry(entryIdentifier: UUID) {
        viewModel.deleteTrustedNetwork(entryIdentifier: entryIdentifier)

        updateCellFactory(with: viewModel)
        updateSnapshot(animated: true)

        reload(item: .useTrustedNetworks)
    }

    func didChangeTrustedNetworkEntry(with identifier: UUID, newSsid: String) {
        viewModel.updateTrustedNetwork(entryIdentifier: identifier, newSsid: newSsid)

        updateCellFactory(with: viewModel)
        updateSnapshot()

        reload(item: .useTrustedNetworks)
    }
}
