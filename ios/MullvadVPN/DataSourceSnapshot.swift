//
//  DataSourceSnapshot.swift
//  MullvadVPN
//
//  Created by pronebird on 11/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct DataSourceSnapshot<Section: Hashable, Item: Hashable> {
    private var _sections = NSMutableOrderedSet()
    private var _items = [Section: NSMutableOrderedSet]()

    var sections: [Section] {
        return _sections.array as! [Section]
    }

    mutating func appendItems(_ itemsToAppend: [Item], in section: Section) {
        assert(_sections.contains(section))
        let items = _items[section] ?? NSMutableOrderedSet()
        items.addObjects(from: itemsToAppend)
        _items[section] = items
    }

    mutating func appendSections(_ newSections: [Section]) {
        _sections.addObjects(from: newSections)
    }

    func section(at index: Int) -> Section? {
        if index < _sections.count {
            let sectionIdentifier = _sections.object(at: index) as! Section
            return sectionIdentifier
        } else {
            return nil
        }
    }

    func indexOfSection(_ section: Section) -> Int? {
        let index = _sections.index(of: section)
        if index == NSNotFound {
            return nil
        } else {
            return index
        }
    }

    func numberOfSections() -> Int {
        return _sections.count
    }

    func numberOfItems(in section: Section) -> Int {
        return _items[section]?.count ?? 0
    }

    func items(in section: Section) -> [Item] {
        if let items = _items[section]?.array {
            return items as! [Item]
        } else {
            return []
        }
    }

    func itemForIndexPath(_ indexPath: IndexPath) -> Item? {
        guard let sectionIdentifier = section(at: indexPath.section),
              let itemSet = _items[sectionIdentifier] else { return nil }

        if indexPath.row < itemSet.count {
            let itemIdentifier = itemSet.object(at: indexPath.row) as! Item

            return itemIdentifier
        } else {
            return nil
        }
    }

    func indexPathForItem(_ item: Item, in section: Section) -> IndexPath? {
        guard let sectionIndex = indexOfSection(section),
              let itemSet = _items[section] else { return nil }

        let itemIndex = itemSet.index(of: item)
        if itemIndex == NSNotFound {
            return nil
        } else {
            return IndexPath(row: itemIndex, section: sectionIndex)
        }
    }
}
