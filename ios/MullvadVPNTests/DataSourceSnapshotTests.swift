//
//  DataSourceSnapshotTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 25/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import XCTest

class DataSourceSnapshotTests: XCTestCase {
    func testInsertingItem() throws {
        var a = DataSourceSnapshot<String, Int>()
        var b = DataSourceSnapshot<String, Int>()

        a.appendSections(["First"])
        b.appendSections(["First"])

        a.appendItems([1, 3], in: "First")
        b.appendItems([1, 2, 3], in: "First")

        let diff = a.difference(b)

        XCTAssertEqual(diff.indexPathsToDelete, [])
        XCTAssertEqual(diff.indexPathsToInsert, [IndexPath(row: 1, section: 0)])
    }

    func testRemovingItem() throws {
        var a = DataSourceSnapshot<String, Int>()
        var b = DataSourceSnapshot<String, Int>()

        a.appendSections(["First"])
        b.appendSections(["First"])

        a.appendItems([1, 2, 3], in: "First")
        b.appendItems([1, 3], in: "First")

        let diff = a.difference(b)

        XCTAssertEqual(diff.indexPathsToDelete, [IndexPath(row: 1, section: 0)])
        XCTAssertEqual(diff.indexPathsToInsert, [])
    }

    func testMovingItemWithinSection() throws {
        var a = DataSourceSnapshot<String, Int>()
        var b = DataSourceSnapshot<String, Int>()

        a.appendSections(["First"])
        b.appendSections(["First"])

        a.appendItems([1, 2, 3], in: "First")
        b.appendItems([2, 1, 3], in: "First")

        let diff = a.difference(b)

        XCTAssertEqual(diff.indexPathsToDelete, [IndexPath(row: 0, section: 0)])
        XCTAssertEqual(diff.indexPathsToInsert, [IndexPath(row: 1, section: 0)])
    }

    func testMovingItemBetweenSections() throws {
        var a = DataSourceSnapshot<String, Int>()
        var b = DataSourceSnapshot<String, Int>()

        a.appendSections(["First", "Second"])
        b.appendSections(["First", "Second"])

        a.appendItems([1, 2, 3, 4], in: "First")
        a.appendItems([5, 6, 7, 8], in: "Second")

        b.appendItems([5, 1, 2, 8, 4], in: "First")
        b.appendItems([6, 3, 7], in: "Second")

        let diff = a.difference(b)

        XCTAssertEqual(diff.indexPathsToDelete, [
            IndexPath(row: 3, section: 1),
            IndexPath(row: 0, section: 1),
            IndexPath(row: 2, section: 0),
        ])

        XCTAssertEqual(diff.indexPathsToInsert, [
            IndexPath(row: 0, section: 0),
            IndexPath(row: 3, section: 0),
            IndexPath(row: 1, section: 1),
        ])
    }

    func testSwappingItems() throws {
        var a = DataSourceSnapshot<String, Int>()
        var b = DataSourceSnapshot<String, Int>()

        a.appendSections(["First"])
        b.appendSections(["First"])

        a.appendItems([1, 2, 3], in: "First")
        b.appendItems([3, 2, 1], in: "First")

        let diff = a.difference(b)

        XCTAssertEqual(diff.indexPathsToDelete, [
            IndexPath(row: 2, section: 0),
            IndexPath(row: 0, section: 0),
        ])

        XCTAssertEqual(diff.indexPathsToInsert, [
            IndexPath(row: 0, section: 0),
            IndexPath(row: 2, section: 0),
        ])
    }

    func testShiftingItems() throws {
        var a = DataSourceSnapshot<String, Int>()
        var b = DataSourceSnapshot<String, Int>()

        a.appendSections(["First"])
        b.appendSections(["First"])

        a.appendItems([1, 2, 3, 4], in: "First")
        b.appendItems([1, 3, 4, 5], in: "First")

        let diff = a.difference(b)

        XCTAssertEqual(diff.indexPathsToDelete, [IndexPath(row: 1, section: 0)])
        XCTAssertEqual(diff.indexPathsToInsert, [IndexPath(row: 3, section: 0)])
    }

    func testReloadingAndReconfiguringItems() throws {
        var a = DataSourceSnapshot<String, Int>()
        var b = DataSourceSnapshot<String, Int>()

        a.appendSections(["First"])
        b.appendSections(["First"])

        a.appendItems([1, 2], in: "First")
        b.appendItems([1, 2], in: "First")

        b.reloadItems([1])
        b.reconfigureItems([2])

        let diff = a.difference(b)

        XCTAssertEqual(diff.indexPathsToReload, [IndexPath(row: 0, section: 0)])
        XCTAssertEqual(diff.indexPathsToReconfigure, [IndexPath(row: 1, section: 0)])
    }
}
