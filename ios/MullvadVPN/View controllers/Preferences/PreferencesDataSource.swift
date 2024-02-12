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
    typealias InfoButtonHandler = (Item) -> Void

    enum CellReuseIdentifiers: String, CaseIterable {
        case dnsSettings
        case wireGuardPort
        case wireGuardCustomPort
        case wireGuardObfuscation
        case wireGuardObfuscationPort
        case quantumResistance
        var reusableViewClass: AnyClass {
            switch self {
            case .dnsSettings:
                return SettingsCell.self
            case .wireGuardPort:
                return SelectableSettingsCell.self
            case .wireGuardCustomPort:
                return SettingsInputCell.self
            case .wireGuardObfuscation:
                return SelectableSettingsCell.self
            case .wireGuardObfuscationPort:
                return SelectableSettingsCell.self
            case .quantumResistance:
                return SelectableSettingsCell.self
            }
        }
    }

    private enum HeaderFooterReuseIdentifiers: String, CaseIterable {
        case wireGuardPortHeader

        var reusableViewClass: AnyClass {
            return SettingsHeaderView.self
        }
    }

    enum Section: String, Hashable, CaseIterable {
        case dnsSettings
        case wireGuardPorts
        case wireGuardObfuscation
        case wireGuardObfuscationPort
        #if DEBUG
        case quantumResistance
        #endif
    }

    enum Item: Hashable {
        case dnsSettings
        case wireGuardPort(_ port: UInt16?)
        case wireGuardCustomPort
        case wireGuardObfuscationAutomatic
        case wireGuardObfuscationOn
        case wireGuardObfuscationOff
        case wireGuardObfuscationPort(_ port: UInt16)
        #if DEBUG
        case quantumResistanceAutomatic
        case quantumResistanceOn
        case quantumResistanceOff
        #endif

        static var wireGuardPorts: [Item] {
            let defaultPorts = PreferencesViewModel.defaultWireGuardPorts.map {
                Item.wireGuardPort($0)
            }
            return [.wireGuardPort(nil)] + defaultPorts + [.wireGuardCustomPort]
        }

        static var wireGuardObfuscation: [Item] {
            [.wireGuardObfuscationAutomatic, .wireGuardObfuscationOn, wireGuardObfuscationOff]
        }

        static var wireGuardObfuscationPort: [Item] {
            [.wireGuardObfuscationPort(0), wireGuardObfuscationPort(80), wireGuardObfuscationPort(5001)]
        }

        #if DEBUG
        static var quantumResistance: [Item] {
            [.quantumResistanceAutomatic, .quantumResistanceOn, .quantumResistanceOff]
        }
        #endif

        var accessibilityIdentifier: AccessibilityIdentifier {
            switch self {
            case .dnsSettings:
                return .dnsSettings
            case .wireGuardPort:
                return .wireGuardPort
            case .wireGuardCustomPort:
                return .wireGuardCustomPort
            case .wireGuardObfuscationAutomatic:
                return .wireGuardObfuscationAutomatic
            case .wireGuardObfuscationOn:
                return .wireGuardObfuscationOn
            case .wireGuardObfuscationOff:
                return .wireGuardObfuscationOff
            case .wireGuardObfuscationPort:
                return .wireGuardObfuscationAutomatic
            #if DEBUG
            case .quantumResistanceAutomatic:
                return .quantumResistanceAutomatic
            case .quantumResistanceOn:
                return .quantumResistanceOn
            case .quantumResistanceOff:
                return .quantumResistanceOff
            #endif
            }
        }

        var reuseIdentifier: CellReuseIdentifiers {
            switch self {
            case .dnsSettings:
                return .dnsSettings
            case .wireGuardPort:
                return .wireGuardPort
            case .wireGuardCustomPort:
                return .wireGuardCustomPort
            case .wireGuardObfuscationAutomatic, .wireGuardObfuscationOn, .wireGuardObfuscationOff:
                return .wireGuardObfuscation
            case .wireGuardObfuscationPort:
                return .wireGuardObfuscationPort
            #if DEBUG
            case .quantumResistanceAutomatic, .quantumResistanceOn, .quantumResistanceOff:
                return .quantumResistance
            #endif
            }
        }
    }

    private(set) var viewModel = PreferencesViewModel() { didSet {
        preferencesCellFactory.viewModel = viewModel
    }}
    private let preferencesCellFactory: PreferencesCellFactory
    private weak var tableView: UITableView?

    weak var delegate: PreferencesDataSourceDelegate?

    var selectedIndexPaths: [IndexPath] {
        let wireGuardPortItem: Item = viewModel.customWireGuardPort == nil
            ? .wireGuardPort(viewModel.wireGuardPort)
            : .wireGuardCustomPort

        let obfuscationStateItem: Item = switch viewModel.obfuscationState {
        case .automatic: .wireGuardObfuscationAutomatic
        case .off: .wireGuardObfuscationOff
        case .on: .wireGuardObfuscationOn
        }
        #if DEBUG
        let quantumResistanceItem: Item = switch viewModel.quantumResistance {
        case .automatic: .quantumResistanceAutomatic
        case .off: .quantumResistanceOff
        case .on: .quantumResistanceOn
        }
        #endif

        let obfuscationPortItem: Item = .wireGuardObfuscationPort(viewModel.obfuscationPort.portValue)

        #if DEBUG
        return [
            wireGuardPortItem,
            obfuscationStateItem,
            obfuscationPortItem,
            quantumResistanceItem,
        ].compactMap { indexPath(for: $0) }
        #else
        return [
            wireGuardPortItem,
            obfuscationStateItem,
            obfuscationPortItem,
        ].compactMap { indexPath(for: $0) }
        #endif
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

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, willDisplay cell: UITableViewCell, forRowAt indexPath: IndexPath) {
        if selectedIndexPaths.contains(indexPath) {
            cell.setSelected(true, animated: false)
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let item = itemIdentifier(for: indexPath)

        deselectAllRowsInSectionExceptRowAt(indexPath)

        switch item {
        case .dnsSettings:
            tableView.deselectRow(at: indexPath, animated: false)
            delegate?.showDNSSettings()

        case let .wireGuardPort(port):
            viewModel.setWireGuardPort(port)

            if let cell = getCustomPortCell() {
                cell.reset()
                cell.textField.resignFirstResponder()
            }

            delegate?.didSelectWireGuardPort(port)

        case .wireGuardCustomPort:
            getCustomPortCell()?.textField.becomeFirstResponder()

        case .wireGuardObfuscationAutomatic:
            selectObfuscationState(.automatic)
            delegate?.didChangeViewModel(viewModel)
        case .wireGuardObfuscationOn:
            selectObfuscationState(.on)
            delegate?.didChangeViewModel(viewModel)
        case .wireGuardObfuscationOff:
            selectObfuscationState(.off)
            delegate?.didChangeViewModel(viewModel)
        case let .wireGuardObfuscationPort(port):
            selectObfuscationPort(port)
            delegate?.didChangeViewModel(viewModel)

        #if DEBUG
        case .quantumResistanceAutomatic:
            selectQuantumResistance(.automatic)
            delegate?.didChangeViewModel(viewModel)
        case .quantumResistanceOn:
            selectQuantumResistance(.on)
            delegate?.didChangeViewModel(viewModel)
        case .quantumResistanceOff:
            selectQuantumResistance(.off)
            delegate?.didChangeViewModel(viewModel)
        #endif
        default:
            break
        }
    }

    // Disallow selection for tapping on a selectable setting.
    func tableView(_ tableView: UITableView, willDeselectRowAt indexPath: IndexPath) -> IndexPath? {
        let item = itemIdentifier(for: indexPath)

        switch item {
        case .dnsSettings:
            return indexPath
        default:
            return nil
        }
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        guard let view = tableView
            .dequeueReusableHeaderFooterView(
                withIdentifier: HeaderFooterReuseIdentifiers.wireGuardPortHeader
                    .rawValue
            ) as? SettingsHeaderView else { return nil }

        switch sectionIdentifier {
        case .wireGuardPorts:
            configureWireguardPortsHeader(view)
            return view

        case .wireGuardObfuscation:
            configureObfuscationHeader(view)
            return view
        case .wireGuardObfuscationPort:
            configureObfuscationPortHeader(view)
            return view
        #if DEBUG
        case .quantumResistance:
            configureQuantumResistanceHeader(view)
            return view
        #endif

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
        case .dnsSettings:
            return 0

        default:
            return UITableView.automaticDimension
        }
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        switch sectionIdentifier {
        case .dnsSettings:
            return 10
        default:
            return 0.5
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

        tableView?.register(
            SettingsHeaderView.self,
            forHeaderFooterViewReuseIdentifier: HeaderFooterReuseIdentifiers.wireGuardPortHeader.rawValue
        )
    }

    private func deselectAllRowsInSectionExceptRowAt(_ indexPath: IndexPath) {
        guard let indexPaths = tableView?.indexPathsForSelectedRows else { return }
        let rowsToDeselect = indexPaths.filter { $0.section == indexPath.section && $0.row != indexPath.row }

        rowsToDeselect.forEach {
            tableView?.deselectRow(at: $0, animated: false)
        }
    }

    private func updateSnapshot(animated: Bool = false, completion: (() -> Void)? = nil) {
        var snapshot = NSDiffableDataSourceSnapshot<Section, Item>()

        snapshot.appendSections(Section.allCases)
        snapshot.appendItems([.dnsSettings], toSection: .dnsSettings)

        applySnapshot(snapshot, animated: animated, completion: completion)
    }

    private func applySnapshot(
        _ snapshot: NSDiffableDataSourceSnapshot<Section, Item>,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        apply(snapshot, animatingDifferences: animated) { [weak self] in
            self?.selectedIndexPaths.forEach { self?.selectRow(at: $0) }
            completion?()
        }
    }

    private func reload(item: Item) {
        if let indexPath = indexPath(for: item),
           let cell = tableView?.cellForRow(at: indexPath) {
            preferencesCellFactory.configureCell(cell, item: item, indexPath: indexPath)
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
                self.delegate?.showInfo(for: .wireGuardPorts)
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
            self.map { $0.delegate?.showInfo(for: .wireGuardObfuscation) }
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
            self.map { $0.delegate?.showInfo(for: .wireGuardObfuscationPort) }
        }
    }

    #if DEBUG
    private func configureQuantumResistanceHeader(_ header: SettingsHeaderView) {
        let title = NSLocalizedString(
            "QUANTUM_RESISTANCE_HEADER_LABEL",
            tableName: "Preferences",
            value: "Quantum Resistance",
            comment: ""
        )

        header.titleLabel.text = title
        header.accessibilityCustomActionName = title
        header.didCollapseHandler = { [weak self] header in
            guard let self else { return }

            var snapshot = snapshot()
            if header.isExpanded {
                snapshot.deleteItems(Item.quantumResistance)
            } else {
                snapshot.appendItems(Item.quantumResistance, toSection: .quantumResistance)
            }
            header.isExpanded.toggle()
            applySnapshot(snapshot, animated: true)
        }

        header.infoButtonHandler = { [weak self] in
            self.map { $0.delegate?.showInfo(for: .quantumResistance) }
        }
    }
    #endif

    private func selectRow(at indexPath: IndexPath?, animated: Bool = false) {
        tableView?.selectRow(at: indexPath, animated: animated, scrollPosition: .none)
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
    func showInfo(for button: PreferencesInfoButtonItem) {
        delegate?.showInfo(for: button)
    }

    func addCustomPort(_ port: UInt16) {
        viewModel.setWireGuardPort(port)
        delegate?.didSelectWireGuardPort(port)
    }

    func selectCustomPortEntry(_ port: UInt16) -> Bool {
        if let indexPath = indexPath(for: .wireGuardCustomPort), !selectedIndexPaths.contains(indexPath) {
            deselectAllRowsInSectionExceptRowAt(indexPath)
            selectRow(at: indexPath)
        }

        return viewModel.isPortWithinValidWireGuardRanges(port)
    }

    func selectObfuscationState(_ state: WireGuardObfuscationState) {
        viewModel.setWireGuardObfuscationState(state)
    }

    func selectObfuscationPort(_ port: UInt16) {
        let selectedPort = WireGuardObfuscationPort(rawValue: port)!
        viewModel.setWireGuardObfuscationPort(selectedPort)
    }

    func selectQuantumResistance(_ state: TunnelQuantumResistance) {
        viewModel.setQuantumResistance(state)
    }
}

// swiftlint:disable:this file_length
