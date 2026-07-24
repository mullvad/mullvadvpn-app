//
//  LocationNodeTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-02-28.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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

    // Guards against sorting by raw Unicode scalar value, which would place accented names
    // such as "Áustria" and "África do Sul" after "Zimbabwe" ("Á"/"á" outrank "z" by scalar
    // value). Acute accents fold to their base letter across Latin locales, so this holds
    // regardless of the test host's locale.
    func testSortingIsLocaleAwareForAccentedNames() {
        let names = ["Zimbabwe", "Áustria", "África do Sul", "Bélgica", "Albânia"]
        let nodes = names.map { LocationNode(name: $0, code: $0, showsChildren: false) }

        XCTAssertEqual(
            nodes.sorted().map(\.name),
            ["África do Sul", "Albânia", "Áustria", "Bélgica", "Zimbabwe"]
        )
    }

    // Documents that collation is locale-specific: "Å" sorts among the A's in English but as a
    // distinct letter after "Z" in Swedish. LocationNode's `<` follows whichever applies to
    // Locale.current at runtime; here we pin the locale explicitly since the operator cannot.
    func testAringOrderingIsLocaleSpecific() {
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
