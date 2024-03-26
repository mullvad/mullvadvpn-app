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

class AddLocationsDataSource: UITableViewDiffableDataSource<LocationSection, LocationCellViewModel> {
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
                withIdentifier: LocationSection.allCases[indexPath.section],
                for: indexPath
                // swiftlint:disable:next force_cast
            ) as! LocationCell
            cell.configure(item: itemIdentifier, behavior: .add)
            cell.selectionStyle = .none
            return cell
        }

        tableView.delegate = self
        tableView.registerReusableViews(from: LocationSection.self)
        defaultRowAnimation = .fade
        reloadWithSelectedLocations()
    }

    private func reloadWithSelectedLocations() {
        var locationsList: [LocationCellViewModel] = []
        nodes.forEach { node in
            let viewModel = LocationCellViewModel(
                section: .customLists,
                node: node,
                isSelected: customListLocationNode.children.contains(node)
            )
            locationsList.append(viewModel)

            // Determine if the node should be expanded.
            guard isLocationInCustomList(node: node) else {
                return
            }

            // Walk tree backwards to determine which nodes should be expanded.
            node.forEachAncestor { node in
                node.showsChildren = true
            }

            locationsList.append(contentsOf: recursivelyCreateCellViewModelTree(
                for: node,
                in: .customLists,
                indentationLevel: 1
            ))
        }
        updateDataSnapshot(with: locationsList)
    }

    private func updateDataSnapshot(
        with list: [LocationCellViewModel],
        animated: Bool = false,
        completion: (() -> Void)? = nil
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<LocationSection, LocationCellViewModel>()

        snapshot.appendSections([.customLists])
        snapshot.appendItems(list, toSection: .customLists)

        apply(snapshot, animatingDifferences: animated, completion: completion)
    }

    private func recursivelyCreateCellViewModelTree(
        for node: LocationNode,
        in section: LocationSection,
        indentationLevel: Int
    ) -> [LocationCellViewModel] {
        var viewModels = [LocationCellViewModel]()
        for childNode in node.children {
            viewModels.append(
                LocationCellViewModel(
                    section: .customLists,
                    node: childNode,
                    indentationLevel: indentationLevel,
                    isSelected: customListLocationNode.children.contains(childNode)
                )
            )

            let indentationLevel = indentationLevel + 1

            // Walk tree forward to determine which nodes should be expanded.
            if isLocationInCustomList(node: childNode) {
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

    private func isLocationInCustomList(node: LocationNode) -> Bool {
        customListLocationNode.children.contains(where: { containsChild(parent: node, child: $0) })
    }

    private func containsChild(parent: LocationNode, child: LocationNode) -> Bool {
        parent.flattened.contains(child)
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        // swiftlint:disable:next force_cast
        let cell = super.tableView(tableView, cellForRowAt: indexPath) as! LocationCell
        cell.delegate = self
        return cell
    }
}

extension AddLocationsDataSource: UITableViewDelegate {
    func tableView(_ tableView: UITableView, indentationLevelForRowAt indexPath: IndexPath) -> Int {
        itemIdentifier(for: indexPath)?.indentationLevel ?? 0
    }
}

extension AddLocationsDataSource: LocationCellDelegate {
    func toggleExpanding(cell: LocationCell) {
        guard let indexPath = tableView.indexPath(for: cell),
              let item = itemIdentifier(for: indexPath) else { return }
        let isExpanded = item.node.showsChildren

        item.node.showsChildren = !isExpanded

        var locationList = snapshot().itemIdentifiers

        if !isExpanded {
            locationList.addSubNodes(from: item, at: indexPath)
        } else {
            locationList.removeSubNodes(from: item.node)
        }

        updateDataSnapshot(with: locationList, animated: true, completion: {
            self.scroll(to: item, animated: true)
        })
    }

    func toggleSelection(cell: LocationCell) {
        guard let index = tableView.indexPath(for: cell)?.row else { return }

        var locationList = snapshot().itemIdentifiers
        let item = locationList[index]
        let isSelected = !item.isSelected
        locationList[index].isSelected = isSelected

        locationList.deselectAncestors(from: item.node)
        locationList.toggleSelectionSubNodes(from: item.node, isSelected: isSelected)

        if isSelected {
            customListLocationNode.add(selectedLocation: item.node)
        } else {
            customListLocationNode.remove(selectedLocation: item.node, with: locationList)
        }
        updateDataSnapshot(with: locationList, completion: {
            self.didUpdateCustomList?(self.customListLocationNode)
        })
    }
}

extension AddLocationsDataSource {
    private func scroll(to item: LocationCellViewModel, animated: Bool) {
        guard
            let visibleIndexPaths = tableView.indexPathsForVisibleRows,
            let indexPath = indexPath(for: item)
        else { return }

        if item.node.children.count > visibleIndexPaths.count {
            tableView.scrollToRow(at: indexPath, at: .top, animated: animated)
        } else {
            if let last = item.node.children.last {
                if let lastInsertedIndexPath = self.indexPath(for: LocationCellViewModel(
                    section: .customLists,
                    node: last
                )),
                    let lastVisibleIndexPath = visibleIndexPaths.last,
                    lastInsertedIndexPath >= lastVisibleIndexPath {
                    tableView.scrollToRow(at: lastInsertedIndexPath, at: .bottom, animated: animated)
                }
            }
        }
    }
}

// MARK: - Toggle selection in table view

fileprivate extension [LocationCellViewModel] {
    mutating func deselectAncestors(from node: LocationNode?) {
        node?.forEachAncestor { parent in
            guard let index = firstIndex(where: { $0.node == parent }) else {
                return
            }
            self[index].isSelected = false
        }
    }

    mutating func toggleSelectionSubNodes(from node: LocationNode, isSelected: Bool) {
        node.forEachDescendant { child in
            guard let index = firstIndex(where: { $0.node == child }) else {
                return
            }
            self[index].isSelected = isSelected
        }
    }
}

// MARK: - Update custom list

fileprivate extension CustomListLocationNode {
    func remove(selectedLocation: LocationNode, with locationList: [LocationCellViewModel]) {
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
        node.forEachDescendant { child in
            // removing children if they are already added to custom list
            if let index = children.firstIndex(of: child) {
                children.remove(at: index)
            }
        }
    }

    private func removeAncestors(node: LocationNode) {
        node.forEachAncestor { parent in
            if let index = children.firstIndex(of: parent) {
                children.remove(at: index)
            }
        }
    }

    private func addSiblings(from locationList: [LocationCellViewModel], for node: LocationNode) {
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
