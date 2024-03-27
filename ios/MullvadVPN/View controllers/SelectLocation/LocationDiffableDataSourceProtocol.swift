//
//  LocationDiffableDataSourceProtocol.swift
//  MullvadVPNUITests
//
//  Created by Jon Petersson on 2024-03-27.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol LocationDiffableDataSourceProtocol: UITableViewDiffableDataSource<LocationSection, LocationCellViewModel> {
    var tableView: UITableView { get }
    var sections: [LocationSection] { get }
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
                if let lastInsertedIndexPath = self.indexPath(for: LocationCellViewModel(
                    section: sections[indexPath.section],
                    node: last
                )),
                   let lastVisibleIndexPath = visibleIndexPaths.last,
                   lastInsertedIndexPath >= lastVisibleIndexPath {
                    tableView.scrollToRow(at: lastInsertedIndexPath, at: .bottom, animated: animated)
                }
            }
        }
    }

    func toggle(cell: LocationCell) {
        guard let indexPath = tableView.indexPath(for: cell),
              let item = itemIdentifier(for: indexPath) else { return }

        let sections = sections
        let section = sections[indexPath.section]
        let isExpanded = item.node.showsChildren
        var locationList = snapshot().itemIdentifiers(inSection: section)

        item.node.showsChildren = !isExpanded

        if !isExpanded {
            locationList.addSubNodes(from: item, at: indexPath)
        } else {
            locationList.removeSubNodes(from: item.node)
        }

        let list = sections.enumerated().map { index, section in
            index == indexPath.section
            ? locationList
            : snapshot().itemIdentifiers(inSection: section)
        }

        updateDataSnapshot(with: list, reloadExisting: true, completion: {
            self.scroll(
                to: item,
                animated: true
            )
        })
    }

    func updateDataSnapshot(
        with list: [[LocationCellViewModel]],
        reloadExisting: Bool = false,
        animated: Bool = false,
        completion: (() -> Void)? = nil
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<LocationSection, LocationCellViewModel>()
        let sections = sections

        snapshot.appendSections(sections)
        for (index, section) in sections.enumerated() {
            let items = list[index]

            snapshot.appendItems(items, toSection: section)

            if reloadExisting {
                snapshot.reconfigureOrReloadItems(items)
            }
        }

        apply(snapshot, animatingDifferences: animated, completion: completion)
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
                    indentationLevel: indentationLevel
                )
            )

            if childNode.showsChildren {
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
