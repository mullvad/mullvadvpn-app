//
//  VPNSettingsDataSource.swift
//  MullvadVPN
//
//  Created by pronebird on 05/10/2021.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

final class VPNSettingsDataSource: UITableViewDiffableDataSource<
    VPNSettingsDataSource.Section,
    VPNSettingsDataSource.Item
>, UITableViewDelegate {
    typealias InfoButtonHandler = (Item) -> Void

    enum CellReuseIdentifiers: String, CaseIterable {
        case localNetworkSharing
        case dnsSettings
        case ipOverrides
        case wireGuardPort
        case wireGuardCustomPort
        case wireGuardObfuscation
        case wireGuardObfuscationOption
        case wireGuardObfuscationPort
        case quantumResistance
        case includeAllNetworks

        var reusableViewClass: AnyClass {
            switch self {
            case .localNetworkSharing:
                return SettingsSwitchCell.self
            case .dnsSettings:
                return SettingsCell.self
            case .ipOverrides:
                return SettingsCell.self
            case .wireGuardPort:
                return SelectableSettingsCell.self
            case .wireGuardCustomPort:
                return SettingsInputCell.self
            case .wireGuardObfuscationOption:
                return SelectableSettingsDetailsCell.self
            case .wireGuardObfuscation:
                return SelectableSettingsCell.self
            case .wireGuardObfuscationPort:
                return SelectableSettingsCell.self
            case .quantumResistance:
                return SelectableSettingsCell.self
            case .includeAllNetworks:
                return SettingsSwitchCell.self
            }
        }
    }

    private enum HeaderFooterReuseIdentifiers: String, CaseIterable {
        case settingsHeaderView

        var reusableViewClass: AnyClass {
            return SettingsHeaderView.self
        }
    }

    enum Section: String, Hashable, CaseIterable {
        #if DEBUG
        case localNetworkSharing
        #endif
        case dnsSettings
        case ipOverrides
        case wireGuardPorts
        case wireGuardObfuscation
        case quantumResistance
        case privacyAndSecurity
    }

    enum Item: Hashable {
        case includeAllNetworks(_ enabled: Bool)
        case localNetworkSharing(_ enabled: Bool)
        case dnsSettings
        case ipOverrides
        case wireGuardPort(_ port: UInt16?)
        case wireGuardCustomPort
        case wireGuardObfuscationAutomatic
        case wireGuardObfuscationUdpOverTcp
        case wireGuardObfuscationShadowsocks
        case wireGuardObfuscationOff
        case wireGuardObfuscationPort(_ port: WireGuardObfuscationUdpOverTcpPort)
        case quantumResistanceAutomatic
        case quantumResistanceOn
        case quantumResistanceOff

        static var wireGuardPorts: [Item] {
            let defaultPorts = VPNSettingsViewModel.defaultWireGuardPorts.map {
                Item.wireGuardPort($0)
            }
            return [.wireGuardPort(nil)] + defaultPorts + [.wireGuardCustomPort]
        }

        static var wireGuardObfuscation: [Item] {
            [
                .wireGuardObfuscationAutomatic,
                .wireGuardObfuscationShadowsocks,
                .wireGuardObfuscationUdpOverTcp,
                .wireGuardObfuscationOff,
            ]
        }

        static var wireGuardObfuscationPort: [Item] {
            [
                .wireGuardObfuscationPort(.automatic),
                .wireGuardObfuscationPort(.port80),
                .wireGuardObfuscationPort(.port5001),
            ]
        }

        static var quantumResistance: [Item] {
            [.quantumResistanceAutomatic, .quantumResistanceOn, .quantumResistanceOff]
        }

        var accessibilityIdentifier: AccessibilityIdentifier {
            switch self {
            case .includeAllNetworks:
                return .includeAllNetworks
            case .localNetworkSharing:
                return .localNetworkSharing
            case .dnsSettings:
                return .dnsSettings
            case .ipOverrides:
                return .ipOverrides
            case let .wireGuardPort(port):
                return .wireGuardPort(port)
            case .wireGuardCustomPort:
                return .wireGuardCustomPort
            case .wireGuardObfuscationAutomatic:
                return .wireGuardObfuscationAutomatic
            case .wireGuardObfuscationUdpOverTcp:
                return .wireGuardObfuscationUdpOverTcp
            case .wireGuardObfuscationShadowsocks:
                return .wireGuardObfuscationShadowsocks
            case .wireGuardObfuscationOff:
                return .wireGuardObfuscationOff
            case .wireGuardObfuscationPort:
                return .wireGuardObfuscationPort
            case .quantumResistanceAutomatic:
                return .quantumResistanceAutomatic
            case .quantumResistanceOn:
                return .quantumResistanceOn
            case .quantumResistanceOff:
                return .quantumResistanceOff
            }
        }

        var reuseIdentifier: CellReuseIdentifiers {
            switch self {
            case .includeAllNetworks:
                return .includeAllNetworks
            case .localNetworkSharing:
                return .localNetworkSharing
            case .dnsSettings:
                return .dnsSettings
            case .ipOverrides:
                return .ipOverrides
            case .wireGuardPort:
                return .wireGuardPort
            case .wireGuardCustomPort:
                return .wireGuardCustomPort
            case .wireGuardObfuscationAutomatic, .wireGuardObfuscationOff:
                return .wireGuardObfuscation
            case .wireGuardObfuscationUdpOverTcp, .wireGuardObfuscationShadowsocks:
                return .wireGuardObfuscationOption
            case .wireGuardObfuscationPort:
                return .wireGuardObfuscationPort
            case .quantumResistanceAutomatic, .quantumResistanceOn, .quantumResistanceOff:
                return .quantumResistance
            }
        }
    }

    private(set) var viewModel = VPNSettingsViewModel() { didSet {
        vpnSettingsCellFactory.viewModel = viewModel
    }}
    private let vpnSettingsCellFactory: VPNSettingsCellFactory
    private weak var tableView: UITableView?

    private var obfuscationSettings: WireGuardObfuscationSettings {
        WireGuardObfuscationSettings(
            state: viewModel.obfuscationState,
            udpOverTcpPort: viewModel.obfuscationUpdOverTcpPort,
            shadowsocksPort: viewModel.obfuscationShadowsocksPort
        )
    }

    weak var delegate: VPNSettingsDataSourceDelegate?

    var selectedIndexPaths: [IndexPath] {
        var wireGuardPortItem: Item = .wireGuardPort(viewModel.wireGuardPort)
        if let customPort = indexPath(for: .wireGuardCustomPort) {
            if tableView?.indexPathsForSelectedRows?.contains(customPort) ?? false {
                wireGuardPortItem = .wireGuardCustomPort
            }
        }

        let obfuscationStateItem: Item = switch viewModel.obfuscationState {
        case .automatic: .wireGuardObfuscationAutomatic
        case .off: .wireGuardObfuscationOff
        case .on, .udpOverTcp: .wireGuardObfuscationUdpOverTcp
        case .shadowsocks: .wireGuardObfuscationShadowsocks
        }

        let quantumResistanceItem: Item = switch viewModel.quantumResistance {
        case .automatic: .quantumResistanceAutomatic
        case .off: .quantumResistanceOff
        case .on: .quantumResistanceOn
        }

        let obfuscationPortItem: Item = .wireGuardObfuscationPort(viewModel.obfuscationUpdOverTcpPort)

        return [
            wireGuardPortItem,
            obfuscationStateItem,
            obfuscationPortItem,
            quantumResistanceItem,
        ].compactMap { indexPath(for: $0) }
    }

    init(tableView: UITableView) {
        self.tableView = tableView

        let vpnSettingsCellFactory = VPNSettingsCellFactory(
            tableView: tableView,
            viewModel: viewModel
        )
        self.vpnSettingsCellFactory = vpnSettingsCellFactory

        super.init(tableView: tableView) { _, indexPath, itemIdentifier in
            vpnSettingsCellFactory.makeCell(for: itemIdentifier, indexPath: indexPath)
        }

        tableView.delegate = self
        vpnSettingsCellFactory.delegate = self

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
                    if let indexPath = indexPath(for: item) {
                        deselectAllRowsInSectionExceptRowAt(indexPath)
                    }
                    selectRow(at: item)
                    return
                }
            }
        }
    }

    func update(from tunnelSettings: LatestTunnelSettings) {
        updateViewModel(from: tunnelSettings)
        updateSnapshot()
    }

    func reload(from tunnelSettings: LatestTunnelSettings) {
        updateViewModel(from: tunnelSettings)

        var snapshot = snapshot()
        snapshot.reconfigureItems(snapshot.itemIdentifiers)
        applySnapshot(snapshot, animated: false)
    }

    // MARK: - UITableViewDelegate

    func tableView(_ tableView: UITableView, willDisplay cell: UITableViewCell, forRowAt indexPath: IndexPath) {
        if selectedIndexPaths.contains(indexPath) {
            cell.setSelected(true, animated: false)
        }
    }

    func tableView(_ tableView: UITableView, willSelectRowAt indexPath: IndexPath) -> IndexPath? {
        let item = itemIdentifier(for: indexPath)

        switch item {
        case .includeAllNetworks, .localNetworkSharing:
            return nil
        default: return indexPath
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let item = itemIdentifier(for: indexPath)

        deselectAllRowsInSectionExceptRowAt(indexPath)

        switch item {
        case .dnsSettings:
            tableView.deselectRow(at: indexPath, animated: false)
            delegate?.showDNSSettings()

        case .ipOverrides:
            tableView.deselectRow(at: indexPath, animated: false)
            delegate?.showIPOverrides()

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
            delegate?.didUpdateTunnelSettings(TunnelSettingsUpdate.obfuscation(obfuscationSettings))
        case .wireGuardObfuscationUdpOverTcp:
            selectObfuscationState(.udpOverTcp)
            delegate?.didUpdateTunnelSettings(TunnelSettingsUpdate.obfuscation(obfuscationSettings))
        case .wireGuardObfuscationShadowsocks:
            selectObfuscationState(.shadowsocks)
            delegate?.didUpdateTunnelSettings(TunnelSettingsUpdate.obfuscation(obfuscationSettings))
        case .wireGuardObfuscationOff:
            selectObfuscationState(.off)
            delegate?.didUpdateTunnelSettings(TunnelSettingsUpdate.obfuscation(obfuscationSettings))
        case let .wireGuardObfuscationPort(port):
            selectObfuscationPort(port)
            delegate?.didUpdateTunnelSettings(TunnelSettingsUpdate.obfuscation(obfuscationSettings))
        case .quantumResistanceAutomatic:
            selectQuantumResistance(.automatic)
            delegate?.didUpdateTunnelSettings(TunnelSettingsUpdate.quantumResistance(viewModel.quantumResistance))
        case .quantumResistanceOn:
            selectQuantumResistance(.on)
            delegate?.didUpdateTunnelSettings(TunnelSettingsUpdate.quantumResistance(viewModel.quantumResistance))
        case .quantumResistanceOff:
            selectQuantumResistance(.off)
            delegate?.didUpdateTunnelSettings(TunnelSettingsUpdate.quantumResistance(viewModel.quantumResistance))
        default:
            break
        }
    }

    // Disallow selection for tapping on a selectable setting.
    func tableView(_ tableView: UITableView, willDeselectRowAt indexPath: IndexPath) -> IndexPath? {
        let item = itemIdentifier(for: indexPath)

        switch item {
        case .dnsSettings, .ipOverrides:
            return indexPath
        default:
            return nil
        }
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        guard let view = tableView
            .dequeueReusableHeaderFooterView(
                withIdentifier: HeaderFooterReuseIdentifiers.settingsHeaderView.rawValue
            ) as? SettingsHeaderView else { return nil }

        switch sectionIdentifier {
        case .wireGuardPorts:
            configureWireguardPortsHeader(view)
            return view

        case .wireGuardObfuscation:
            configureObfuscationHeader(view)
            return view
        case .quantumResistance:
            configureQuantumResistanceHeader(view)
            return view
        default:
            return nil
        }
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        let view = UIView()
        view.backgroundColor = tableView.backgroundColor
        return view
    }

    func tableView(_ tableView: UITableView, heightForHeaderInSection section: Int) -> CGFloat {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        switch sectionIdentifier {
        case .dnsSettings, .ipOverrides, .privacyAndSecurity:
            return .leastNonzeroMagnitude
        #if DEBUG
        case .localNetworkSharing: return .leastNonzeroMagnitude
        #endif
        default:
            return tableView.estimatedRowHeight
        }
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        let sectionIdentifier = snapshot().sectionIdentifiers[section]

        return switch sectionIdentifier {
        // 0 due to there already being a separator between .dnsSettings and .ipOverrides.
        case .dnsSettings: 0
        case .ipOverrides, .quantumResistance: UITableView.automaticDimension
        #if DEBUG
        case .localNetworkSharing: UITableView.automaticDimension
        #endif
        default: 0.5
        }
    }

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        let sectionIdentifier = snapshot().sectionIdentifiers[indexPath.section]

        return switch sectionIdentifier {
        case .privacyAndSecurity: false
        default: true
        }
    }

    // MARK: - Private

    func updateViewModel(from tunnelSettings: LatestTunnelSettings) {
        let newViewModel = VPNSettingsViewModel(from: tunnelSettings)
        let mergedViewModel = viewModel.merged(newViewModel)

        if viewModel != mergedViewModel {
            viewModel = mergedViewModel
        }
    }

    private func registerClasses() {
        CellReuseIdentifiers.allCases.forEach { enumCase in
            tableView?.register(
                enumCase.reusableViewClass,
                forCellReuseIdentifier: enumCase.rawValue
            )
        }

        tableView?.register(
            SettingsHeaderView.self,
            forHeaderFooterViewReuseIdentifier: HeaderFooterReuseIdentifiers.settingsHeaderView.rawValue
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
        snapshot.appendItems([.ipOverrides], toSection: .ipOverrides)
        #if DEBUG
        snapshot.appendItems(
            [.localNetworkSharing(viewModel.localNetworkSharing)],
            toSection: .localNetworkSharing
        )
        snapshot
            .appendItems(
                [.includeAllNetworks(viewModel.includeAllNetworks)],
                toSection: .localNetworkSharing
            )
        #endif

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
            vpnSettingsCellFactory.configureCell(cell, item: item, indexPath: indexPath)
        }
    }

    private func configureWireguardPortsHeader(_ header: SettingsHeaderView) {
        let title = NSLocalizedString(
            "WIREGUARD_PORTS_HEADER_LABEL",
            tableName: "VPNSettings",
            value: "WireGuard ports",
            comment: ""
        )

        header.setAccessibilityIdentifier(.wireGuardPortsCell)
        header.titleLabel.text = title
        header.accessibilityCustomActionName = title
        header.isExpanded = isExpanded(.wireGuardPorts)
        header.infoButtonHandler = { [weak self] in
            if let self, let humanReadablePortRepresentation = delegate?.humanReadablePortRepresentation() {
                self.delegate?.showInfo(for: .wireGuardPorts(humanReadablePortRepresentation))
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
            tableName: "VPNSettings",
            value: "WireGuard Obfuscation",
            comment: ""
        )

        header.setAccessibilityIdentifier(.wireGuardObfuscationCell)
        header.titleLabel.text = title
        header.accessibilityCustomActionName = title
        header.isExpanded = isExpanded(.wireGuardObfuscation)
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

    private func configureQuantumResistanceHeader(_ header: SettingsHeaderView) {
        let title = NSLocalizedString(
            "QUANTUM_RESISTANCE_HEADER_LABEL",
            tableName: "VPNSettings",
            value: "Quantum-resistant tunnel",
            comment: ""
        )

        header.setAccessibilityIdentifier(.quantumResistantTunnelCell)
        header.titleLabel.text = title
        header.accessibilityCustomActionName = title
        header.isExpanded = isExpanded(.quantumResistance)
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

    /*
      Since we are dequeuing headers, it's crucial to maintain the state of expansion.
      Using screenshots as a single source of truth to capture the state allows us to
      determine whether headers are expanded or not.
     */
    private func isExpanded(_ section: Section) -> Bool {
        let snapshot = snapshot()
        return snapshot.numberOfItems(inSection: section) != 0
    }
}

extension VPNSettingsDataSource: @preconcurrency VPNSettingsCellEventHandler {
    func setIncludeAllNetworks(_ enabled: Bool, onCancel: @escaping () -> Void) {
        self.viewModel.setIncludeAllNetworks(enabled)
        self.delegate?.didUpdateTunnelSettings(.includeAllNetworks(enabled))
    }

    func setLocalNetworkSharing(_ enabled: Bool, onCancel: @escaping () -> Void) {
        delegate?.showLocalNetworkSharingWarning(enabled) { accepted in
            if accepted {
                self.delegate?.didUpdateTunnelSettings(.localNetworkSharing(enabled))
                self.viewModel.setLocalNetworkSharing(enabled)
            } else {
                onCancel()
            }
        }
    }

    func showInfo(for button: VPNSettingsInfoButtonItem) {
        delegate?.showInfo(for: button)
    }

    func showDetails(for button: VPNSettingsDetailsButtonItem) {
        delegate?.showDetails(for: button)
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

    func selectObfuscationPort(_ port: WireGuardObfuscationUdpOverTcpPort) {
        viewModel.setWireGuardObfuscationUdpOverTcpPort(port)
    }

    func selectQuantumResistance(_ state: TunnelQuantumResistance) {
        viewModel.setQuantumResistance(state)
    }

    func switchMultihop(_ state: MultihopState) {
        viewModel.setMultihop(state)
        delegate?.didUpdateTunnelSettings(.multihop(viewModel.multihopState))
    }
}

// swiftlint:disable:this file_length
