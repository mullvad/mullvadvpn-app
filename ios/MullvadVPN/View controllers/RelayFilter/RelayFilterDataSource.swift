//
//  RelayFilterDataSource.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import RelayCache
import UIKit

final class RelayFilterDataSource: UITableViewDiffableDataSource<
    RelayFilterDataSource.Section,
    RelayFilterDataSource.Item
> {
    private var tableView: UITableView?
    private let relayFilterCellFactory: RelayFilterCellFactory
    private var viewModel: RelayFilterViewModel

    var selectedOwnershipItem: Item {
        guard let selectedIndexPath = getSelectedIndexPaths(in: .ownership).first,
              let selectedItem = itemIdentifier(for: selectedIndexPath)
        else {
            return .ownershipAny
        }

        return selectedItem
    }

    var selectedProviderItems: [Item] {
        return getSelectedIndexPaths(in: .providers).compactMap { indexPath in
            itemIdentifier(for: indexPath)
        }
    }

    init(tableView: UITableView, viewModel: RelayFilterViewModel) {
        self.tableView = tableView
        self.viewModel = viewModel

        let relayFilterCellFactory = RelayFilterCellFactory(tableView: tableView)
        self.relayFilterCellFactory = relayFilterCellFactory

        super.init(tableView: tableView) { tableView, indexPath, itemIdentifier in
            relayFilterCellFactory.makeCell(for: itemIdentifier, indexPath: indexPath)
        }

        tableView.delegate = self

        viewModel.didUpdateRelays = { [weak self] in
            self?.updateDataSnapshot()
        }

        viewModel.didUpdateFilter.append { [weak self] _ in
            self?.updateDataSnapshot()
        }

        registerClasses()
        createDataSnapshot()
    }

    func tableView(_ tableView: UITableView, willDisplay cell: UITableViewCell, forRowAt indexPath: IndexPath) {
        switch getSection(for: indexPath) {
        case .ownership:
            if viewModel.getOwnership(for: itemIdentifier(for: indexPath)) == viewModel.relayFilter.ownership {
                cell.setSelected(true, animated: false)
            }
        case .providers:
            switch viewModel.relayFilter.providers {
            case .any:
                cell.setSelected(true, animated: false)
            case let .only(providers):
                switch itemIdentifier(for: indexPath) {
                case .allProviders:
                    let allProvidersAreSelected = providers.count == viewModel.uniqueProviders.count
                    if allProvidersAreSelected {
                        cell.setSelected(true, animated: false)
                    }
                case let .provider(name):
                    if providers.contains(name) {
                        cell.setSelected(true, animated: false)
                    }
                default:
                    break
                }
            }
        }
    }

    private func registerClasses() {
        CellReuseIdentifiers.allCases.forEach { cellIdentifier in
            tableView?.register(
                cellIdentifier.reusableViewClass,
                forCellReuseIdentifier: cellIdentifier.rawValue
            )
        }

        HeaderFooterReuseIdentifiers.allCases.forEach { reuseIdentifier in
            tableView?.register(
                reuseIdentifier.reusableViewClass,
                forHeaderFooterViewReuseIdentifier: reuseIdentifier.rawValue
            )
        }
    }

    private func createDataSnapshot() {
        var snapshot = NSDiffableDataSourceSnapshot<Section, Item>()
        snapshot.appendSections(Section.allCases)

        applySnapshot(snapshot, animated: false)
    }

    private func updateDataSnapshot() {
        let oldSnapshot = snapshot()

        var newSnapshot = NSDiffableDataSourceSnapshot<Section, Item>()
        newSnapshot.appendSections(Section.allCases)

        Section.allCases.forEach { section in
            switch section {
            case .ownership:
                if !oldSnapshot.itemIdentifiers(inSection: section).isEmpty {
                    newSnapshot.appendItems(Item.ownerships, toSection: .ownership)
                }
            case .providers:
                if !oldSnapshot.itemIdentifiers(inSection: section).isEmpty {
                    let items = viewModel.uniqueProviders.map { Item.provider($0) }

                    newSnapshot.appendItems([.allProviders])
                    newSnapshot.appendItems(items, toSection: .providers)
                }
            }
        }

        applySnapshot(newSnapshot, animated: false)
    }

    private func applySnapshot(
        _ snapshot: NSDiffableDataSourceSnapshot<Section, Item>,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        apply(snapshot, animatingDifferences: animated) { [weak self] in
            guard let self else { return }

            updateSelection(from: viewModel.relayFilter)
            completion?()
        }
    }

    private func updateSelection(from filter: RelayFilter) {
        if let ownershipItem = viewModel.getOwnershipItem(for: filter.ownership) {
            selectRow(true, at: indexPath(for: ownershipItem))
        }

        switch filter.providers {
        case .any:
            selectAllProviders(true)
        case let .only(providers):
            providers.forEach { providerName in
                if let providerItem = viewModel.getProviderItem(for: providerName) {
                    selectRow(true, at: indexPath(for: providerItem))
                }
            }

            updateAllProvidersSelection()
        }
    }

    private func updateAllProvidersSelection() {
        if viewModel.uniqueProviders.count == getSelectedIndexPaths(in: .providers).count {
            selectRow(true, at: indexPath(for: .allProviders))
        }
    }
}

extension RelayFilterDataSource: UITableViewDelegate {
    func tableView(_ tableView: UITableView, willSelectRowAt indexPath: IndexPath) -> IndexPath? {
        switch getSection(for: indexPath) {
        case .ownership:
            if let selectedIndexPath = self.indexPath(for: selectedOwnershipItem) {
                selectRow(false, at: selectedIndexPath)
            }
        case .providers:
            break
        }

        return indexPath
    }

    func tableView(_ tableView: UITableView, willDeselectRowAt indexPath: IndexPath) -> IndexPath? {
        switch getSection(for: indexPath) {
        case .ownership:
            return nil
        case .providers:
            return indexPath
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let item = itemIdentifier(for: indexPath) else { return }

        switch getSection(for: indexPath) {
        case .ownership:
            break
        case .providers:
            if item == .allProviders {
                selectAllProviders(true)
            } else {
                updateAllProvidersSelection()
            }
        }

        viewModel.addItemToFilter(item)
    }

    func tableView(_ tableView: UITableView, didDeselectRowAt indexPath: IndexPath) {
        guard let item = itemIdentifier(for: indexPath) else { return }

        switch getSection(for: indexPath) {
        case .ownership:
            break
        case .providers:
            if item == .allProviders {
                selectAllProviders(false)
            } else {
                selectRow(false, at: self.indexPath(for: .allProviders))
            }
        }

        viewModel.removeItemFromFilter(item)
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        guard let view = tableView.dequeueReusableHeaderFooterView(
            withIdentifier: HeaderFooterReuseIdentifiers.section.rawValue
        ) as? SettingsHeaderView else { return nil }

        let sectionId = snapshot().sectionIdentifiers[section]
        let title: String

        switch sectionId {
        case .ownership:
            title = "Ownership"
        case .providers:
            title = "Providers"
        }

        view.titleLabel.text = NSLocalizedString(
            "RELAY_FILTER_HEADER_LABEL",
            tableName: "Relay filter header",
            value: title,
            comment: ""
        )

        view.didCollapseHandler = { [weak self] headerView in
            guard let self else { return }

            var snapshot = snapshot()

            switch sectionId {
            case .ownership:
                handleCollapseOwnership(snapshot: &snapshot, isExpanded: headerView.isExpanded)
            case .providers:
                handleCollapseProviders(snapshot: &snapshot, isExpanded: headerView.isExpanded)
            }

            headerView.isExpanded.toggle()
            applySnapshot(snapshot, animated: true)
        }

        return view
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        return nil
    }

    func tableView(_ tableView: UITableView, heightForHeaderInSection section: Int) -> CGFloat {
        return UITableView.automaticDimension
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        return UIMetrics.TableView.separatorHeight
    }

    private func selectRow(_ select: Bool, at indexPath: IndexPath?) {
        guard let indexPath else { return }

        if select {
            tableView?.selectRow(at: indexPath, animated: false, scrollPosition: .none)
        } else {
            tableView?.deselectRow(at: indexPath, animated: false)
        }
    }

    private func getSelectedIndexPaths(in section: Section) -> [IndexPath] {
        let sectionIndex = snapshot().indexOfSection(section)

        return tableView?.indexPathsForSelectedRows?.filter { indexPath in
            indexPath.section == sectionIndex
        } ?? []
    }

    private func getSection(for indexPath: IndexPath) -> Section {
        return snapshot().sectionIdentifiers[indexPath.section]
    }

    private func selectAllProviders(_ select: Bool) {
        let providerItems = snapshot().itemIdentifiers(inSection: .providers)

        providerItems.forEach { providerItem in
            selectRow(select, at: indexPath(for: providerItem))
        }
    }

    private func handleCollapseOwnership(
        snapshot: inout NSDiffableDataSourceSnapshot<RelayFilterDataSource.Section, RelayFilterDataSource.Item>,
        isExpanded: Bool
    ) {
        if isExpanded {
            snapshot.deleteItems(Item.ownerships)
        } else {
            snapshot.appendItems(Item.ownerships, toSection: .ownership)
        }
    }

    private func handleCollapseProviders(
        snapshot: inout NSDiffableDataSourceSnapshot<RelayFilterDataSource.Section, RelayFilterDataSource.Item>,
        isExpanded: Bool
    ) {
        if isExpanded {
            let items = snapshot.itemIdentifiers(inSection: .providers)
            snapshot.deleteItems(items)
        } else {
            let items = viewModel.uniqueProviders.map { Item.provider($0) }

            snapshot.appendItems([.allProviders])
            snapshot.appendItems(items, toSection: .providers)
        }
    }
}

extension RelayFilterDataSource {
    enum CellReuseIdentifiers: String, CaseIterable {
        case ownershipCell
        case providerCell

        var reusableViewClass: AnyClass {
            switch self {
            case .ownershipCell:
                return SelectableSettingsCell.self
            case .providerCell:
                return CheckableSettingsCell.self
            }
        }
    }

    enum HeaderFooterReuseIdentifiers: String, CaseIterable {
        case section

        var reusableViewClass: AnyClass {
            return SettingsHeaderView.self
        }
    }

    enum Section: Hashable, CaseIterable {
        case ownership
        case providers
    }

    enum Item: Hashable {
        case ownershipAny
        case ownershipOwned
        case ownershipRented
        case allProviders
        case provider(_ name: String)

        static var ownerships: [Item] {
            return [.ownershipAny, .ownershipOwned, .ownershipRented]
        }

        var reuseIdentifier: CellReuseIdentifiers {
            switch self {
            case .ownershipAny, .ownershipOwned, .ownershipRented:
                return .ownershipCell
            case .allProviders, .provider:
                return .providerCell
            }
        }
    }
}
