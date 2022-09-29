//
//  DataSourceSnapshot.swift
//  MullvadVPN
//
//  Created by pronebird on 11/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

/// `NSDiffableDataSourceSnapshot` replica.
struct DataSourceSnapshot<Section: Hashable, Item: Hashable> {
    /// Ordered set of section identifiers.
    private var orderedSections = NSMutableOrderedSet()

    /// Ordered set of item identifiers.
    private var orderedItems = NSMutableOrderedSet()

    /// Item identifier ranges by section.
    private var sectionToItemMapping = [Range<Int>]()

    /// Items to reload.
    private var itemsToReload = NSMutableOrderedSet()

    /// Items to reconfigure.
    private var itemsToReconfigure = NSMutableOrderedSet()

    /// Ordered array of section identifiers.
    var sectionIdentifiers: [Section] {
        return orderedSections.array as! [Section]
    }

    /// Ordered array of item identifiers.
    var itemIdentifiers: [Item] {
        return orderedItems.array as! [Item]
    }

    mutating func appendItems(_ itemsToAppend: [Item], in section: Section) {
        assert(orderedSections.contains(section))

        let sectionIndex = indexOfSection(section)!
        let uniqueItemsToAppend = NSOrderedSet(array: itemsToAppend)
        let itemRange = sectionToItemMapping[sectionIndex]

        let oldEndIndex = itemRange.endIndex
        let newEndIndex = itemRange.endIndex.advanced(by: uniqueItemsToAppend.count)
        let newItemRange = (itemRange.startIndex ..< newEndIndex)

        sectionToItemMapping[sectionIndex] = newItemRange
        orderedItems.insert(
            uniqueItemsToAppend.array,
            at: IndexSet(integersIn: oldEndIndex ..< newEndIndex)
        )

        offsetItemRange(inSectionsAfter: sectionIndex, by: uniqueItemsToAppend.count)
    }

    mutating func appendSections(_ newSections: [Section]) {
        let lastSectionRange = sectionToItemMapping.last
        let emptyRange = lastSectionRange.flatMap { range in
            return (range.upperBound ..< range.upperBound)
        } ?? (0 ..< 0)

        let uniqueNewSections = NSOrderedSet(array: newSections)

        for newSection in uniqueNewSections {
            orderedSections.add(newSection)
            sectionToItemMapping.append(emptyRange)
        }
    }

    func section(at index: Int) -> Section? {
        if index < orderedSections.count {
            return orderedSections.object(at: index) as? Section
        } else {
            return nil
        }
    }

    func indexOfSection(_ section: Section) -> Int? {
        let index = orderedSections.index(of: section)
        if index == NSNotFound {
            return nil
        } else {
            return index
        }
    }

    func numberOfSections() -> Int {
        return orderedSections.count
    }

    func numberOfItems(in section: Section) -> Int? {
        guard let sectionIndex = indexOfSection(section) else { return nil }

        return sectionToItemMapping[sectionIndex].count
    }

    func items(in section: Section) -> [Item] {
        guard let sectionIndex = indexOfSection(section) else { return [] }

        let range = sectionToItemMapping[sectionIndex]
        let indexSet = IndexSet(integersIn: range)

        return orderedItems.objects(at: indexSet) as! [Item]
    }

    func itemForIndexPath(_ indexPath: IndexPath) -> Item? {
        guard indexPath.section < orderedSections.count else { return nil }

        let itemRange = sectionToItemMapping[indexPath.section]
        let itemIndex = itemRange.startIndex + indexPath.row

        if itemRange.contains(itemIndex) {
            return orderedItems.object(at: itemIndex) as? Item
        } else {
            return nil
        }
    }

    func indexPathForItem(_ item: Item) -> IndexPath? {
        let itemIndex = orderedItems.index(of: item)
        guard itemIndex != NSNotFound else { return nil }

        guard let sectionIdentifier = section(containingItem: item) else { return nil }

        let sectionIndex = orderedSections.index(of: sectionIdentifier)
        guard sectionIndex != NSNotFound else { return nil }

        let range = sectionToItemMapping[sectionIndex]
        let rowIndex = itemIndex - range.startIndex

        return IndexPath(row: rowIndex, section: sectionIndex)
    }

    func section(containingItem item: Item) -> Section? {
        let itemIndex = orderedItems.index(of: item)
        guard itemIndex != NSNotFound else { return nil }

        for (sectionIndex, sectionObject) in orderedSections.enumerated() {
            let sectionIdentifier = sectionObject as! Section
            let range = sectionToItemMapping[sectionIndex]

            if range.contains(itemIndex) {
                return sectionIdentifier
            }
        }

        return nil
    }

    mutating func reloadItems(_ items: [Item]) {
        itemsToReload.addObjects(from: items)
    }

    mutating func reconfigureItems(_ items: [Item]) {
        itemsToReconfigure.addObjects(from: items)
    }

    private mutating func offsetItemRange(inSectionsAfter sectionIndex: Int, by offset: Int) {
        let startIndex = sectionIndex + 1
        let sectionRange = (startIndex ..< orderedSections.count)

        for sectionIndex in sectionRange {
            let range = sectionToItemMapping[sectionIndex]
            let offsetRange = (range.startIndex + offset ..< range.endIndex + offset)

            sectionToItemMapping[sectionIndex] = offsetRange
        }
    }
}

extension DataSourceSnapshot {
    enum Change: CustomDebugStringConvertible, Hashable {
        case insert(IndexPath)
        case delete(IndexPath)
        case move(_ source: IndexPath, _ target: IndexPath)
        case reload(IndexPath)
        case reconfigure(IndexPath)

        var sortOrder: Int {
            switch self {
            case .delete:
                return 0
            case .insert:
                return 1
            case .move:
                return 2
            case .reload:
                return 3
            case .reconfigure:
                return 4
            }
        }

        var debugDescription: String {
            switch self {
            case let .insert(indexPath):
                return "insert \(indexPath)"
            case let .delete(indexPath):
                return "delete \(indexPath)"
            case let .move(source, target):
                return "move from \(source) to \(target)"
            case let .reload(indexPath):
                return "reload \(indexPath)"
            case let .reconfigure(indexPath):
                return "reconfigure \(indexPath)"
            }
        }

        func breakMoveOntoInsertionDeletion() -> [Change] {
            if case let .move(fromIndexPath, toIndexPath) = self {
                return [.delete(fromIndexPath), .insert(toIndexPath)]
            } else {
                return [self]
            }
        }
    }

    func difference(_ other: DataSourceSnapshot<Section, Item>) -> DataSnapshotDifference {
        var changes = [Change]()

        let oldItems = itemIdentifiers
        let newItems = other.itemIdentifiers

        for item in oldItems {
            let oldIndexPath = indexPathForItem(item)
            let newIndexPath = other.indexPathForItem(item)

            if let oldIndexPath = oldIndexPath, oldIndexPath != newIndexPath {
                guard let newIndexPath = newIndexPath else {
                    changes.append(.delete(oldIndexPath))
                    continue
                }

                // Guard against recording the `.move` twice when exchanging two adjacent items.
                let isSwappingTwoAdjacentItems = changes.contains { otherChange in
                    if case let .move(fromIndexPath, toIndexPath) = otherChange {
                        let itemDistance = abs(oldIndexPath.row - fromIndexPath.row)

                        return oldIndexPath == toIndexPath && newIndexPath == fromIndexPath &&
                            oldIndexPath.section == newIndexPath.section &&
                            itemDistance == 1

                    } else {
                        return false
                    }
                }

                if !isSwappingTwoAdjacentItems {
                    changes.append(.move(oldIndexPath, newIndexPath))
                }
            }
        }

        for item in newItems {
            if let indexPath = other.indexPathForItem(item), !oldItems.contains(item) {
                changes.append(.insert(indexPath))
            }
        }

        changes = Self.inferMoves(changes: changes)

        for itemObject in other.itemsToReload {
            let itemIdentifier = itemObject as! Item
            if let indexPath = other.indexPathForItem(itemIdentifier) {
                changes.append(.reload(indexPath))
            }
        }

        for itemObject in other.itemsToReconfigure {
            let itemIdentifier = itemObject as! Item
            if let indexPath = other.indexPathForItem(itemIdentifier) {
                changes.append(.reconfigure(indexPath))
            }
        }

        changes.sort(by: Self.changeSortPredicate)

        return Self.changeSetToDifference(changes)
    }

    /// Infer and discard unnecessary moves that occur due to items shifting back or forth based on
    /// insertions and deletions of other items.
    private static func inferMoves(changes: [Change]) -> [Change] {
        var newChanges = [Change]()

        // Expand .move onto .insert + .delete pair and sort changes.
        let sortedChangesWithoutMoves = changes
            .flatMap { change in
                return change.breakMoveOntoInsertionDeletion()
            }
            .sorted(by: Self.changeSortPredicate)

        for sourceChange in changes {
            guard case let .move(sourceIndexPath, targetIndexPath) = sourceChange else {
                newChanges.append(sourceChange)
                continue
            }

            // Replay all changes to compute the item's index path, ignoring the changes
            // associated with the current change.
            let inferredIndexPath = sortedChangesWithoutMoves
                .reduce(into: sourceIndexPath) { inferredIndexPath, otherChange in
                    switch otherChange {
                    case let .insert(insertedIndexPath) where insertedIndexPath != targetIndexPath:
                        if inferredIndexPath.row >= insertedIndexPath.row,
                           inferredIndexPath.section == insertedIndexPath.section
                        {
                            inferredIndexPath.row += 1
                        }

                    case let .delete(deletedIndexPath) where deletedIndexPath != sourceIndexPath:
                        if inferredIndexPath.row > deletedIndexPath.row,
                           inferredIndexPath.section == deletedIndexPath.section
                        {
                            inferredIndexPath.row -= 1
                        }

                    default:
                        break
                    }
                }

            // Discard the change if the index path, produced after replaying other changes,
            // matches the target index path.
            if inferredIndexPath != targetIndexPath {
                newChanges.append(contentsOf: sourceChange.breakMoveOntoInsertionDeletion())
            }
        }

        return newChanges
    }

    /// Sort predicate used for sorting a collection of `Change`.
    ///
    /// Sort order by kind and index path:
    /// Deletion: descending
    /// Insertion: ascending
    /// Reload, reconfigure: ascending
    private static func changeSortPredicate(_ lhs: Change, _ rhs: Change) -> Bool {
        switch (lhs, rhs) {
        case let (.insert(lhsIndexPath), .insert(rhsIndexPath)):
            return lhsIndexPath < rhsIndexPath

        case let (.delete(lhsIndexPath), .delete(rhsIndexPath)):
            return lhsIndexPath > rhsIndexPath

        case let (.reload(lhsIndexPath), .reload(rhsIndexPath)):
            return lhsIndexPath < rhsIndexPath

        case let (.reconfigure(lhsIndexPath), .reconfigure(rhsIndexPath)):
            return lhsIndexPath < rhsIndexPath

        case let (lhs, rhs):
            return lhs.sortOrder < rhs.sortOrder
        }
    }

    private static func changeSetToDifference(_ changes: [Change]) -> DataSnapshotDifference {
        var indexPathsToInsert = [IndexPath]()
        var indexPathsToDelete = [IndexPath]()
        var indexPathsToReload = [IndexPath]()
        var indexPathsToReconfigure = [IndexPath]()

        for change in changes {
            switch change {
            case let .insert(indexPath):
                indexPathsToInsert.append(indexPath)

            case let .delete(indexPath):
                indexPathsToDelete.append(indexPath)

            case .move:
                // Moves are broken down onto insert and delete changes at this point.
                break

            case let .reload(indexPath):
                indexPathsToReload.append(indexPath)

            case let .reconfigure(indexPath):
                indexPathsToReconfigure.append(indexPath)
            }
        }

        return DataSnapshotDifference(
            indexPathsToInsert: indexPathsToInsert,
            indexPathsToDelete: indexPathsToDelete,
            indexPathsToReload: indexPathsToReload,
            indexPathsToReconfigure: indexPathsToReconfigure
        )
    }
}

struct StackViewApplyDataSnapshotConfiguration {
    var animationDuration: TimeInterval = 0.25
    var animationOptions: UIView.AnimationOptions = [.curveEaseInOut]
    var makeView: (IndexPath) -> UIView
}

struct DataSnapshotDifference: CustomDebugStringConvertible {
    var indexPathsToInsert = [IndexPath]()
    var indexPathsToDelete = [IndexPath]()
    var indexPathsToReload = [IndexPath]()
    var indexPathsToReconfigure = [IndexPath]()

    var debugDescription: String {
        var s = "DataSnapshotDifference {\n"

        s += "  insert: \n"
        for indexPath in indexPathsToInsert {
            s += "    \(indexPath),\n"
        }

        s += "  delete: \n"
        for indexPath in indexPathsToDelete {
            s += "    \(indexPath),\n"
        }

        s += "  reload: \n"
        for indexPath in indexPathsToReload {
            s += "    \(indexPath),\n"
        }

        s += "  reconfigure: \n"
        for indexPath in indexPathsToReconfigure {
            s += "    \(indexPath),\n"
        }

        s += "}"

        return s
    }

    func apply(
        to tableView: UITableView,
        animateDifferences: Bool,
        completion: ((Bool) -> Void)? = nil
    ) {
        let animation: UITableView.RowAnimation = animateDifferences ? .automatic : .none

        tableView.performBatchUpdates({
            if !indexPathsToDelete.isEmpty {
                tableView.deleteRows(at: indexPathsToDelete, with: animation)
            }

            if !indexPathsToInsert.isEmpty {
                tableView.insertRows(at: indexPathsToInsert, with: animation)
            }

            if !indexPathsToReload.isEmpty {
                tableView.reloadRows(at: indexPathsToReload, with: animation)
            }

            if !indexPathsToReconfigure.isEmpty {
                if #available(iOS 15.0, *) {
                    tableView.reconfigureRows(at: indexPathsToReconfigure)
                } else {
                    tableView.reloadRows(at: indexPathsToReconfigure, with: .none)
                }
            }
        }, completion: completion)
    }

    func apply(
        to stackView: UIStackView,
        configuration: StackViewApplyDataSnapshotConfiguration,
        animateDifferences: Bool,
        completion: ((Bool) -> Void)? = nil
    ) {
        let viewsToRemove = indexPathsToDelete.map { indexPath in
            return stackView.arrangedSubviews[indexPath.row]
        }

        let viewsToAdd = indexPathsToInsert.map { indexPath -> UIView in
            let view = configuration.makeView(indexPath)

            view.isHidden = true
            view.alpha = 0

            var viewIndex = indexPath.row

            // Adjust insertion index since views are not removed from stack view during animation.
            for view in stackView.arrangedSubviews[..<indexPath.row] {
                if viewsToRemove.contains(view) {
                    viewIndex += 1
                }
            }

            stackView.insertArrangedSubview(view, at: viewIndex)

            return view
        }

        // Layout inserted subviews before running animations to achieve a folding effect.
        if animateDifferences {
            UIView.performWithoutAnimation {
                stackView.layoutIfNeeded()
            }
        }

        let showHideViews = {
            for view in viewsToRemove {
                view.alpha = 0
                view.isHidden = true
            }

            for view in viewsToAdd {
                view.alpha = 1
                view.isHidden = false
            }
        }

        let removeViews = {
            for view in viewsToRemove {
                view.removeFromSuperview()
            }
        }

        if animateDifferences {
            UIView.animate(
                withDuration: configuration.animationDuration,
                delay: 0,
                options: configuration.animationOptions,
                animations: {
                    showHideViews()
                    stackView.layoutIfNeeded()
                },
                completion: { isComplete in
                    removeViews()
                    completion?(isComplete)
                }
            )
        } else {
            showHideViews()
            removeViews()
            completion?(true)
        }
    }
}
