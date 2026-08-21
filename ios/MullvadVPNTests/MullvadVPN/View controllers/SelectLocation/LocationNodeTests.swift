// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import XCTest

class LocationNodeTests: XCTestCase {
    let listNode = CustomListLocationNode(
        name: "List",
        code: "list",
        showsChildren: false,
        customList: CustomList(name: "List", locations: [])
    )
    let countryNode = LocationNode(name: "Country", code: "country", showsChildren: false)
    let cityNode = LocationNode(name: "City", code: "city", showsChildren: false)
    let hostNode = LocationNode(name: "Host", code: "host", showsChildren: false)

    override func setUp() async throws {
        createNodeTree()
    }

    func testNodeTree() throws {
        XCTAssertEqual(listNode.children.first, countryNode)
        XCTAssertEqual(countryNode.children.first, cityNode)
        XCTAssertEqual(cityNode.children.first, hostNode)
        XCTAssertNil(hostNode.children.first)
    }

    func testTopmostAncestor() throws {
        XCTAssertEqual(hostNode.root, listNode)
    }

    func testAnscestors() throws {
        hostNode.forEachAncestor { node in
            node.showsChildren = true
        }

        XCTAssertTrue(listNode.showsChildren)
        XCTAssertTrue(countryNode.showsChildren)
        XCTAssertTrue(cityNode.showsChildren)
        XCTAssertFalse(hostNode.showsChildren)
    }

    func testDescendants() throws {
        listNode.forEachDescendant { node in
            node.showsChildren = true
        }

        XCTAssertFalse(listNode.showsChildren)
        XCTAssertTrue(countryNode.showsChildren)
        XCTAssertTrue(cityNode.showsChildren)
        XCTAssertTrue(hostNode.showsChildren)
    }

    func testCopyNode() throws {
        let hostNodeCopy = hostNode.copy()

        XCTAssertTrue(hostNode == hostNodeCopy)
        XCTAssertFalse(hostNode === hostNodeCopy)

        var numberOfDescendants = 0
        hostNode.forEachDescendant { _ in
            numberOfDescendants += 1
        }

        var numberOfCopyDescendants = 0
        hostNodeCopy.forEachDescendant { _ in
            numberOfCopyDescendants += 1
        }

        XCTAssertEqual(numberOfDescendants, numberOfCopyDescendants)
    }

    func testFindByCountryCode() {
        XCTAssertTrue(listNode.countryFor(code: countryNode.code) == countryNode)
    }

    func testFindByCityCode() {
        XCTAssertTrue(countryNode.cityFor(codes: [cityNode.code]) == cityNode)
    }

    func testFindByHostCode() {
        XCTAssertTrue(cityNode.hostFor(code: hostNode.code) == hostNode)
    }

    func testFindDescendantByNodeCode() {
        XCTAssertTrue(listNode.descendantNode(for: [hostNode.code]) == hostNode)
    }

    func testOrderingIsLocaleSpecific() {
        XCTAssertEqual(
            "Åland".compare("Zimbabwe", options: [], range: nil, locale: Locale(identifier: "en_US")), .orderedAscending
        )
        XCTAssertEqual(
            "Åland".compare("Zimbabwe", options: [], range: nil, locale: Locale(identifier: "sv_SE")),
            .orderedDescending)
    }
}

extension LocationNodeTests {
    private func createNodeTree() {
        hostNode.parent = cityNode
        cityNode.children.append(hostNode)

        cityNode.parent = countryNode
        countryNode.children.append(cityNode)

        countryNode.parent = listNode
        listNode.children.append(countryNode)
    }
}
