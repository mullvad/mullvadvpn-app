//
//  AddLocationsDataSource.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes
import UIKit

enum AddLocationsSectionIdentifier: String, Hashable, CaseIterable, CellIdentifierProtocol {
    case `default`

    var cellClass: AnyClass {
        switch self {
        case .default: AddLocationCell.self
        }
    }
}

class AddLocationsDataSource: UITableViewDiffableDataSource<AddLocationsSectionIdentifier, AddLocationCellViewModel> {
    private let tableView: UITableView
    private let nodes: [LocationNode]
    private var customListLocationNode: CustomListLocationNode
    var didUpdateCustomList: ((CustomListLocationNode) -> Void)?

    init(
        tableView: UITableView,
        allLocations: [LocationNode],
        customList: CustomList
    ) {
        self.tableView = tableView
        self.nodes = allLocations

        self.customListLocationNode = CustomListLocationNodeBuilder(
            customList: customList,
            allLocations: self.nodes
        ).customListLocationNode

        super.init(tableView: tableView) { _, indexPath, itemIdentifier in
            let cell = tableView.dequeueReusableView(
                withIdentifier: AddLocationsSectionIdentifier.allCases[indexPath.section],
                for: indexPath
                // swiftlint:disable:next force_cast
            ) as! AddLocationCell
            cell.configure(item: itemIdentifier)
            cell.selectionStyle = .none
            return cell
        }

        tableView.delegate = self
        tableView.registerReusableViews(from: AddLocationsSectionIdentifier.self)
        defaultRowAnimation = .fade
        reloadWithSelectedLocations()
    }

    private func reloadWithSelectedLocations() {
        var locationsList: [AddLocationCellViewModel] = []
        nodes.forEach { node in
            let viewModel = AddLocationCellViewModel(
                node: node,
                isSelected: customListLocationNode.children.contains(node)
            )
            locationsList.append(viewModel)

            // Determine if the node should be expanded.
            guard customListLocationNode.children.contains(where: { node.allChildren.contains($0) }) else {
                return
            }

            // Walk tree backwards to determine which nodes should be expanded.
            node.forEachAncestor { node in
                node.showsChildren = true
            }

            locationsList.append(contentsOf: recursivelyCreateCellViewModelTree(
                for: node,
                in: .default,
                indentationLevel: 1
            ))
        }
        updateDataSnapshot(with: [locationsList], completion: {
            // Scroll to the first selected location
            guard let first = locationsList.first(where: { self.customListLocationNode.children.contains($0.node) })
            else {
                return
            }
            self.scroll(to: first, animated: false)
        })
    }

    private func updateDataSnapshot(
        with list: [[AddLocationCellViewModel]],
        animated: Bool = false,
        completion: (() -> Void)? = nil
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<AddLocationsSectionIdentifier, AddLocationCellViewModel>()
        let sections = AddLocationsSectionIdentifier.allCases

        snapshot.appendSections(sections)

        for (index, section) in sections.enumerated() {
            let items = list[index]

            snapshot.appendItems(items, toSection: section)
        }
        apply(snapshot, animatingDifferences: animated, completion: completion)
    }

    private func recursivelyCreateCellViewModelTree(
        for node: LocationNode,
        in section: AddLocationsSectionIdentifier,
        indentationLevel: Int
    ) -> [AddLocationCellViewModel] {
        var viewModels = [AddLocationCellViewModel]()
        for childNode in node.children {
            viewModels.append(
                AddLocationCellViewModel(
                    node: childNode,
                    indentationLevel: indentationLevel,
                    isSelected: customListLocationNode.children.contains(childNode)
                )
            )

            let indentationLevel = indentationLevel + 1

            // Walk tree forward to determine which nodes should be expanded.
            if customListLocationNode.children.contains(where: { childNode.allChildren.contains($0) }) {
                viewModels.append(
                    contentsOf: recursivelyCreateCellViewModelTree(
                        for: childNode,
                        in: section,
                        indentationLevel: indentationLevel
                    )
                )
            }
        }

        return viewModels
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        // swiftlint:disable:next force_cast
        let cell = super.tableView(tableView, cellForRowAt: indexPath) as! AddLocationCell
        cell.delegate = self
        return cell
    }
}

extension AddLocationsDataSource: UITableViewDelegate {
    func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        itemIdentifier(for: indexPath)?.indentationLevel ?? 0
    }
}

extension AddLocationsDataSource: AddLocationCellDelegate {
    func toggleExpanding(cell: AddLocationCell) {
        guard let indexPath = tableView.indexPath(for: cell),
              let item = itemIdentifier(for: indexPath) else { return }
        let isExpanded = item.node.showsChildren

        item.node.showsChildren = !isExpanded

        var locationList = snapshot().itemIdentifiers

        if !isExpanded {
            locationList.addSubNodes(from: item, at: indexPath)
        } else {
            locationList.recursivelyRemoveSubNodes(from: item.node)
        }

        updateDataSnapshot(with: [locationList], animated: true, completion: {
            self.scroll(to: item, animated: true)
        })
    }

    func toggleSelection(cell: AddLocationCell) {
        guard let index = tableView.indexPath(for: cell)?.row else { return }

        var locationList = snapshot().itemIdentifiers
        let item = locationList[index]
        let isSelected = !item.isSelected
        locationList[index].isSelected = isSelected

        locationList.deselectAncestors(from: item.node.parent)
        locationList.toggleSelectionSubNodes(from: item.node, isSelected: isSelected)

        if isSelected {
            customListLocationNode.add(selectedLocation: item.node)
        } else {
            customListLocationNode.remove(selectedLocation: item.node, with: locationList)
        }
        updateDataSnapshot(with: [locationList], completion: {
            self.didUpdateCustomList?(self.customListLocationNode)
        })
    }
}

extension AddLocationsDataSource {
    private func scroll(to item: AddLocationCellViewModel, animated: Bool) {
        guard
            let visibleIndexPaths = tableView.indexPathsForVisibleRows,
            let indexPath = indexPath(for: item)
        else { return }

        if item.node.children.count > visibleIndexPaths.count {
            tableView.scrollToRow(at: indexPath, at: .top, animated: animated)
        } else {
            if let last = item.node.children.last {
                if let lastInsertedIndexPath = self.indexPath(for: AddLocationCellViewModel(
                    node: last,
                    isSelected: false
                )),
                    let lastVisibleIndexPath = visibleIndexPaths.last,
                    lastInsertedIndexPath >= lastVisibleIndexPath {
                    tableView.scrollToRow(at: lastInsertedIndexPath, at: .bottom, animated: animated)
                }
            }
        }
    }
}

// MARK: - Toggle expanding

fileprivate extension [AddLocationCellViewModel] {
    mutating func addSubNodes(from item: AddLocationCellViewModel, at indexPath: IndexPath) {
        let row = indexPath.row + 1
        let locations = item.node.children.map {
            AddLocationCellViewModel(node: $0, indentationLevel: item.indentationLevel + 1, isSelected: item.isSelected)
        }

        if row < count {
            insert(contentsOf: locations, at: row)
        } else {
            append(contentsOf: locations)
        }
    }

    mutating func recursivelyRemoveSubNodes(from node: LocationNode) {
        for node in node.children {
            node.showsChildren = false
            removeAll(where: { node == $0.node })
            recursivelyRemoveSubNodes(from: node)
        }
    }
}

// MARK: - Toggle selection in table view

fileprivate extension [AddLocationCellViewModel] {
    mutating func deselectAncestors(from node: LocationNode?) {
        guard let index = firstIndex(where: { $0.node == node }) else {
            return
        }
        self[index].isSelected = false
        node?.parent.flatMap {
            deselectAncestors(from: $0)
        }
    }

    mutating func toggleSelectionSubNodes(from node: LocationNode, isSelected: Bool) {
        for node in node.children {
            guard let index = firstIndex(where: { $0.node == node }) else {
                return
            }
            self[index].isSelected = isSelected
            toggleSelectionSubNodes(from: node, isSelected: isSelected)
        }
    }
}

// MARK: - Update custom list

fileprivate extension CustomListLocationNode {
    func remove(selectedLocation: LocationNode, with locationList: [AddLocationCellViewModel]) {
        if let index = children.firstIndex(of: selectedLocation) {
            children.remove(at: index)
        }
        removeAncestors(node: selectedLocation)
        addSiblings(from: locationList, for: selectedLocation)
    }

    func add(selectedLocation: LocationNode) {
        children.append(selectedLocation)
        removeSubNodes(node: selectedLocation)
    }

    private func removeSubNodes(node: LocationNode) {
        for child in node.children {
            // removing children if they are already added to custom list
            if let index = children.firstIndex(of: child) {
                children.remove(at: index)
            }
            removeSubNodes(node: child)
        }
    }

    private func removeAncestors(node: LocationNode) {
        guard let parent = node.parent else { return }
        if let index = children.firstIndex(of: parent) {
            children.remove(at: index)
        }
        removeAncestors(node: parent)
    }

    private func addSiblings(from locationList: [AddLocationCellViewModel], for node: LocationNode) {
        guard let parent = node.parent else { return }
        parent.children.forEach { child in
            // adding siblings if they are already selected in snapshot
            if let item = locationList.first(where: { $0.node == child }),
               item.isSelected && !children.contains(child) {
                children.append(child)
            }
        }
        addSiblings(from: locationList, for: parent)
    }
}
