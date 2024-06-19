//
//  AddLocationsDataSource.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import MullvadTypes
import UIKit

class AddLocationsDataSource:
    UITableViewDiffableDataSource<LocationSection, LocationCellViewModel>,
    LocationDiffableDataSourceProtocol {
    private var customListLocationNode: CustomListLocationNode
    private let nodes: [LocationNode]
    private let subject: CurrentValueSubject<CustomListViewModel, Never>
    let tableView: UITableView
    let sections: [LocationSection]

    init(
        tableView: UITableView,
        allLocationNodes: [LocationNode],
        subject: CurrentValueSubject<CustomListViewModel, Never>
    ) {
        self.tableView = tableView
        self.nodes = allLocationNodes
        self.subject = subject

        self.customListLocationNode = CustomListLocationNodeBuilder(
            customList: subject.value.customList,
            allLocations: self.nodes
        ).customListLocationNode

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

            // Only parents with partially selected children should be expanded.
            node.forEachDescendant { descendantNode in
                if customListLocationNode.children.contains(descendantNode) {
                    descendantNode.forEachAncestor { descendantParentNode in
                        descendantParentNode.showsChildren = true
                    }
                }
            }

            locationsList.append(contentsOf: recursivelyCreateCellViewModelTree(
                for: node,
                in: .customLists,
                indentationLevel: 1
            ))
        }
        updateDataSnapshot(with: [locationsList])
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
        let items = toggledItems(for: cell).first!.map { item in
            var item = item
            if containsChild(parent: customListLocationNode, child: item.node) {
                item.isSelected = true
            }
            return item
        }

        updateDataSnapshot(with: [items], reloadExisting: true, completion: {
            if let indexPath = self.tableView.indexPath(for: cell),
               let item = self.itemIdentifier(for: indexPath) {
                self.scroll(to: item, animated: true)
            }
        })
    }

    func toggleSelecting(cell: LocationCell) {
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
        updateDataSnapshot(with: [locationList], completion: {
            let locations = self.customListLocationNode.children.reduce([]) { partialResult, locationNode in
                partialResult + locationNode.locations
            }
            self.subject.value.locations = locations
        })
    }
}

// MARK: - Called from LocationDiffableDataSourceProtocol

extension AddLocationsDataSource {
    func nodeShowsChildren(_ node: LocationNode) -> Bool {
        isLocationInCustomList(node: node)
    }

    func nodeShouldBeSelected(_ node: LocationNode) -> Bool {
        customListLocationNode.children.contains(node)
    }

    func excludedRelayTitle(_ node: LocationNode) -> String? {
        nil // N/A
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
