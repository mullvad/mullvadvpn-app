//
//  LocationNodeTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-02-28.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import XCTest

class LocationNodeTests: XCTestCase {
    let listNode = ListLocationNode(
        nodeName: "List",
        nodeCode: "list",
        showsChildren: false,
        customList: CustomList(name: "List", locations: [])
    )
    let countryNode = CountryLocationNode(nodeName: "Country", nodeCode: "country", showsChildren: false)
    let cityNode = CityLocationNode(nodeName: "City", nodeCode: "city", showsChildren: false)
    let hostNode = HostLocationNode(nodeName: "Host", nodeCode: "host", showsChildren: false)

    override func setUp() async throws {
        createNodeTree()
    }

    func testTopmostAncestor() throws {
        XCTAssertEqual(hostNode.topmostAncestor, listNode)
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
        XCTAssertTrue(listNode.countryFor(countryCode: countryNode.nodeCode) == countryNode)
    }

    func testFindByCityCode() {
        XCTAssertTrue(countryNode.cityFor(cityCode: cityNode.nodeCode) == cityNode)
    }

    func testFindByHostCode() {
        XCTAssertTrue(cityNode.hostFor(hostCode: hostNode.nodeCode) == hostNode)
    }

    func testFindChildByNodeCode() {
        XCTAssertTrue(listNode.childNodeFor(nodeCode: hostNode.nodeCode) == hostNode)
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
