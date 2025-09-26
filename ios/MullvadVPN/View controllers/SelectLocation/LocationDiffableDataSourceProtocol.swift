//
//  LocationDiffableDataSourceProtocol.swift
//  MullvadVPNUITests
//
//  Created by Jon Petersson on 2024-03-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import UIKit

protocol LocationDiffableDataSourceProtocol: UITableViewDiffableDataSource<LocationSection, LocationCellViewModel> {
    var tableView: UITableView { get }
    var sections: [LocationSection] { get }
    func nodeShowsChildren(_ node: LocationNode) -> Bool
    func nodeShouldBeSelected(_ node: LocationNode) -> Bool
}

extension LocationDiffableDataSourceProtocol {
    func scroll(to item: LocationCellViewModel, animated: Bool) {
        guard
            let visibleIndexPaths = tableView.indexPathsForVisibleRows,
            let indexPath = indexPath(for: item)
        else { return }

        if item.node.children.count > visibleIndexPaths.count {
            tableView.scrollToRow(at: indexPath, at: .top, animated: animated)
        } else {
            if let last = item.node.children.last {
                if let lastInsertedIndexPath = self.indexPath(
                    for: LocationCellViewModel(
                        section: sections[indexPath.section],
                        node: last
                    )),
                    let lastVisibleIndexPath = visibleIndexPaths.last,
                    lastInsertedIndexPath >= lastVisibleIndexPath
                {
                    tableView.scrollToRow(at: lastInsertedIndexPath, at: .bottom, animated: animated)
                }
            }
        }
    }

    func toggleItems(for cell: LocationCell, completion: (() -> Void)? = nil) {
        guard let indexPath = tableView.indexPath(for: cell),
            let item = itemIdentifier(for: indexPath)
        else { return }
        let snapshot = snapshot()
        let section = sections[indexPath.section]
        let isExpanded = item.node.showsChildren
        var locationList = snapshot.itemIdentifiers(inSection: section)

        item.node.showsChildren = !isExpanded

        if !isExpanded {
            locationList.addSubNodes(from: item, at: indexPath)
            addItem(locationList, toSection: section, index: indexPath.row, completion: completion)
        } else {
            locationList.removeSubNodes(from: item.node)
            updateSnapshotRetainingOnly(locationList, toSection: section, completion: completion)
        }
    }

    private func addItem(
        _ items: [LocationCellViewModel],
        toSection section: LocationSection,
        index: Int,
        completion: (() -> Void)? = nil
    ) {
        var snapshot = snapshot()
        let existingItems = snapshot.itemIdentifiers(inSection: section)

        // Filter itemsToAdd to only include items not already in the section
        let uniqueItems = items.filter { item in
            existingItems.firstIndex(where: { $0 == item }) == nil
        }

        // Insert unique items at the specified index
        if index < existingItems.count {
            snapshot.insertItems(uniqueItems, afterItem: existingItems[index])
        } else {
            // If the index is beyond bounds, append to the end
            snapshot.appendItems(uniqueItems, toSection: section)
        }
        applyAndReconfigureSnapshot(snapshot, in: section, completion: completion)
    }

    private func updateSnapshotRetainingOnly(
        _ itemsToKeep: [LocationCellViewModel],
        toSection section: LocationSection,
        completion: (() -> Void)? = nil
    ) {
        var snapshot = snapshot()

        // Ensure the section exists in the snapshot
        guard snapshot.sectionIdentifiers.contains(section) else { return }

        // Get the current items in the section
        let currentItems = snapshot.itemIdentifiers(inSection: section)

        // Determine the items that should be deleted
        let itemsToDelete = currentItems.filter { !itemsToKeep.contains($0) }
        snapshot.deleteItems(itemsToDelete)

        // Apply the updated snapshot
        applyAndReconfigureSnapshot(snapshot, in: section, completion: completion)
    }

    private func applyAndReconfigureSnapshot(
        _ snapshot: NSDiffableDataSourceSnapshot<LocationSection, LocationCellViewModel>,
        in section: LocationSection,
        completion: (() -> Void)? = nil
    ) {
        self.apply(snapshot, animatingDifferences: true) {
            // After adding, reconfigure specified items to update their content
            var updatedSnapshot = self.snapshot()

            // Ensure the items exist in the snapshot before attempting to reconfigure
            let existingItems = updatedSnapshot.itemIdentifiers(inSection: section)

            // Reconfigure the specified items
            updatedSnapshot.reconfigureItems(existingItems)

            // Apply the reconfigured snapshot without animations to avoid any flickering
            self.apply(updatedSnapshot, animatingDifferences: false)
        }
    }

    func reloadDataSnapshot(
        with list: [[LocationCellViewModel]],
        animated: Bool = false,
        completion: (() -> Void)? = nil
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<LocationSection, LocationCellViewModel>()
        snapshot.appendSections(sections)
        for (index, section) in sections.enumerated() {
            let items = list[index]
            snapshot.appendItems(items, toSection: section)
        }
        self.apply(snapshot, animatingDifferences: animated, completion: completion)
    }

    func recursivelyCreateCellViewModelTree(
        for node: LocationNode,
        in section: LocationSection,
        indentationLevel: Int
    ) -> [LocationCellViewModel] {
        var viewModels = [LocationCellViewModel]()

        for childNode in node.children where !childNode.isHiddenFromSearch {
            viewModels.append(
                LocationCellViewModel(
                    section: section,
                    node: childNode,
                    indentationLevel: indentationLevel,
                    isSelected: nodeShouldBeSelected(childNode)
                )
            )

            if nodeShowsChildren(childNode) {
                viewModels.append(
                    contentsOf: recursivelyCreateCellViewModelTree(
                        for: childNode,
                        in: section,
                        indentationLevel: indentationLevel + 1
                    )
                )
            }
        }

        return viewModels
    }
}
