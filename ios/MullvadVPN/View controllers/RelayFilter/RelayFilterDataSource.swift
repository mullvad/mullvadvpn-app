//
//  RelayFilterDataSource.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadTypes
import UIKit
final class RelayFilterDataSource: UITableViewDiffableDataSource<
    RelayFilterDataSource.Section,
    RelayFilterDataSource.Item
> {
    private weak var tableView: UITableView?
    private var viewModel: RelayFilterViewModel
    private let relayFilterCellFactory: RelayFilterCellFactory
    private var disposeBag = Set<Combine.AnyCancellable>()

    init(tableView: UITableView, viewModel: RelayFilterViewModel) {
        self.tableView = tableView
        self.viewModel = viewModel

        let relayFilterCellFactory = RelayFilterCellFactory(tableView: tableView)
        self.relayFilterCellFactory = relayFilterCellFactory

        super.init(tableView: tableView) { _, indexPath, itemIdentifier in
            relayFilterCellFactory.makeCell(for: itemIdentifier, indexPath: indexPath)
        }

        registerCells()
        createDataSnapshot()
        tableView.delegate = self
        setupBindings()
    }

    private func registerCells() {
        CellReuseIdentifiers.allCases.forEach { tableView?.register(
            $0.reusableViewClass,
            forCellReuseIdentifier: $0.rawValue
        ) }
        HeaderFooterReuseIdentifiers.allCases.forEach { tableView?.register(
            $0.reusableViewClass,
            forHeaderFooterViewReuseIdentifier: $0.rawValue
        ) }
    }

    private func setupBindings() {
        viewModel
            .$relayFilter
            .dropFirst()
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] filter in
                self?.updateDataSnapshot(filter: filter)
            }
            .store(in: &disposeBag)
    }

    private func createDataSnapshot() {
        var snapshot = NSDiffableDataSourceSnapshot<Section, Item>()
        snapshot.appendSections(Section.allCases)
        apply(snapshot, animatingDifferences: false)
    }

    private func updateDataSnapshot(filter: RelayFilter) {
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
                    newSnapshot.appendItems(
                        [Item.allProviders] + viewModel.availableProviders(for: filter.ownership),
                        toSection: .providers
                    )
                    applySnapshot(newSnapshot, animated: false)
                }
            }
        }
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
        tableView?.indexPathsForSelectedRows?.forEach { selectRow(false, at: $0) }

        if let ownership = viewModel.ownershipItem(for: filter.ownership),
           let ownershipIndexPath = indexPath(for: ownership) {
            selectRow(true, at: ownershipIndexPath)
        }

        switch filter.providers {
        case .any:
            selectAllProviders(true)
        case let .only(providers):
            selectAllProviders(false)
            providers.forEach { providerName in
                selectRow(true, at: indexPath(for: viewModel.providerItem(for: providerName)))
            }
            updateAllProvidersSelection()
        }
    }

    private func isItemSelected(_ item: Item, for filter: RelayFilter) -> Bool {
        switch item.type {
        case .ownershipAny, .ownershipOwned, .ownershipRented:
            return viewModel.ownership(for: item) == filter.ownership
        case .allProviders:
            return filter.providers == .any
        case .provider:
            return switch filter.providers {
            case .any:
                true
            case let .only(providers):
                providers.contains(item.name)
            }
        }
    }

    private func updateAllProvidersSelection() {
        let selectedCount = getSelectedIndexPaths(in: .providers).count
        let providerCount = viewModel.availableProviders(for: viewModel.relayFilter.ownership).count
        selectRow(selectedCount == providerCount, at: indexPath(for: .allProviders))
    }

    private func handleCollapseOwnership(isExpanded: Bool) {
        var newSnapshot = snapshot()
        if isExpanded {
            newSnapshot.deleteItems(Item.ownerships)
        } else {
            newSnapshot.appendItems(Item.ownerships, toSection: .ownership)
        }
        applySnapshot(newSnapshot, animated: !isExpanded)
    }

    private func handleCollapseProviders(isExpanded: Bool) {
        let currentSnapshot = snapshot()
        var newSnapshot = currentSnapshot

        if isExpanded {
            let items = newSnapshot.itemIdentifiers(inSection: .providers)
            newSnapshot.deleteItems(items)
        } else {
            newSnapshot.appendItems(
                [Item.allProviders] + viewModel.availableProviders(for: viewModel.relayFilter.ownership),
                toSection: .providers
            )
        }
        applySnapshot(newSnapshot, animated: !isExpanded)
    }

    private func selectRow(_ select: Bool, at indexPath: IndexPath?) {
        guard let indexPath else { return }

        if select {
            tableView?.selectRow(at: indexPath, animated: false, scrollPosition: .none)
        } else {
            tableView?.deselectRow(at: indexPath, animated: false)
        }
    }

    private func selectAllProviders(_ select: Bool) {
        let providerItems = snapshot().itemIdentifiers(inSection: .providers)

        providerItems.forEach { providerItem in
            selectRow(select, at: indexPath(for: providerItem))
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
}

// MARK: - UITableViewDelegate

extension RelayFilterDataSource: UITableViewDelegate {
    func tableView(_ tableView: UITableView, willSelectRowAt indexPath: IndexPath) -> IndexPath? {
        switch getSection(for: indexPath) {
        case .ownership:
            selectRow(false, at: getSelectedIndexPaths(in: .ownership).first)
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
        viewModel.toggleItem(item)
    }

    func tableView(_ tableView: UITableView, didDeselectRowAt indexPath: IndexPath) {
        guard let item = itemIdentifier(for: indexPath) else { return }
        viewModel.toggleItem(item)
    }

    func tableView(_ tableView: UITableView, willDisplay cell: UITableViewCell, forRowAt indexPath: IndexPath) {
        guard let item = itemIdentifier(for: indexPath) else { return }
        cell.setSelected(isItemSelected(item, for: viewModel.relayFilter), animated: false)
    }

    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        guard let view = tableView.dequeueReusableHeaderFooterView(
            withIdentifier: HeaderFooterReuseIdentifiers.section.rawValue
        ) as? SettingsHeaderView else { return nil }

        let sectionId = snapshot().sectionIdentifiers[section]
        let title: String
        let accessibilityIdentifier: AccessibilityIdentifier

        switch sectionId {
        case .ownership:
            accessibilityIdentifier = .locationFilterOwnershipHeaderCell
            title = "Ownership"
        case .providers:
            accessibilityIdentifier = .locationFilterProvidersHeaderCell
            title = "Providers"
        }

        view.setAccessibilityIdentifier(accessibilityIdentifier)
        view.titleLabel.text = NSLocalizedString(
            "RELAY_FILTER_HEADER_LABEL",
            tableName: "Relay filter header",
            value: title,
            comment: ""
        )

        view.didCollapseHandler = { [weak self] headerView in
            guard let self else { return }
            switch sectionId {
            case .ownership:
                handleCollapseOwnership(isExpanded: headerView.isExpanded)
            case .providers:
                handleCollapseProviders(isExpanded: headerView.isExpanded)
            }

            headerView.isExpanded.toggle()
        }

        return view
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        return nil
    }

    func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        return UIMetrics.TableView.separatorHeight
    }
}
