//
//  AddLocationsDataSource.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import UIKit

class AddLocationsDataSource:
    UITableViewDiffableDataSource<LocationSection, LocationCellViewModel>,
    LocationDiffableDataSourceProtocol
{
    private let nodes: [LocationNode]
    private var selectedLocations: [RelayLocation]
    var didUpdateLocations: (([RelayLocation]) -> Void)?
    let tableView: UITableView
    let sections: [LocationSection]

    init(
        tableView: UITableView,
        allLocationNodes: [LocationNode],
        selectedLocations: [RelayLocation]
    ) {
        self.tableView = tableView
        self.nodes = allLocationNodes
        self.selectedLocations = selectedLocations

        let sections: [LocationSection] = [.customLists]
        self.sections = sections

        super.init(tableView: tableView) { _, indexPath, itemIdentifier in
            let cell = tableView.dequeueReusableView(
                withIdentifier: sections[indexPath.section],
                for: indexPath
            ) as! LocationCell // swiftlint:disable:this force_cast
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
        var items = nodes.flatMap { node in
            // Create a "faux" root node to use for constructing a node tree.
            let rootNode = RootLocationNode(children: [node])

            // Only parents with partially selected children should be expanded.
            node.forEachDescendant { descendantNode in
                if selectedLocations.containsAny(locations: descendantNode.locations) {
                    descendantNode.parent?.showsChildren = true
                }
            }

            // Construct node tree.
            return recursivelyCreateCellViewModelTree(
                for: rootNode,
                in: .customLists,
                indentationLevel: 0
            )
        }

        // Apply selection to node tree.
        items = items.map { item in
            var item = item
            if selectedLocations.containsAny(locations: item.node.locations) {
                item.isSelected = true
            }
            return item
        }

        updateDataSnapshot(with: [items], reloadExisting: false)
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
        toggle(cell: cell)
    }

    func toggleSelecting(cell: LocationCell) {
        guard let index = tableView.indexPath(for: cell)?.row else { return }

        var items = snapshot().itemIdentifiers
        let item = items[index]

        guard let nodeLocation = item.node.locations.first else { return }

        let isSelected = !item.isSelected
        items[index].isSelected = isSelected

        items.deselectAncestors(from: item.node)
        items.toggleSelectionSubNodes(from: item.node, isSelected: isSelected)

        if isSelected {
            selectedLocations.append(nodeLocation)
        } else {
            selectedLocations.removeAll { $0 == nodeLocation }
        }

        updateDataSnapshot(with: [items], reloadExisting: true, completion: {
            self.didUpdateLocations?(self.selectedLocations)
        })
    }
}

fileprivate extension [RelayLocation] {
    func containsAny(locations: [RelayLocation]) -> Bool {
        locations.contains(where: { location in
            contains(location)
        })
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
