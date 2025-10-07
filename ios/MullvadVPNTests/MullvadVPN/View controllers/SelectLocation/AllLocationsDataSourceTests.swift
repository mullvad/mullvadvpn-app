//
//  AllLocationsDataSourceTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-02-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import XCTest

@testable import MullvadSettings

class AllLocationsDataSourceTests: XCTestCase {
    var allLocationNodes = [LocationNode]()
    var dataSource: AllLocationDataSource!

    override func setUp() async throws {
        setUpDataSource()
    }

    func testNodeTree() throws {
        let rootNode = RootLocationNode(children: dataSource.nodes)

        // Testing a selection.
        XCTAssertNotNil(rootNode.descendantNodeFor(codes: ["se"]))
        XCTAssertNotNil(rootNode.descendantNodeFor(codes: ["us", "dal"]))
        XCTAssertNotNil(rootNode.descendantNodeFor(codes: ["es1-wireguard"]))
        XCTAssertNotNil(rootNode.descendantNodeFor(codes: ["se2-wireguard"]))
    }

    func testSearch() throws {
        let nodes = dataSource.search(by: "got")
        let rootNode = RootLocationNode(children: nodes)

        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se", "got"])?.isHiddenFromSearch == false)
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se", "sto"])?.isHiddenFromSearch == true)
    }

    func testSearch2() throws {
        let nodes = dataSource.search(by: "s")
        let rootNode = RootLocationNode(children: nodes)

        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se", "got"])?.isHiddenFromSearch == false)
        XCTAssertTrue(
            rootNode
                .descendantNodeFor(codes: ["se", "got"])?.showsChildren == true
        )
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se", "sto"])?.isHiddenFromSearch == false)
        XCTAssertTrue(
            rootNode
                .descendantNodeFor(codes: ["se", "sto"])?.showsChildren == true
        )
        XCTAssertTrue(
            rootNode.descendantNodeFor(codes: ["se"])?.showsChildren == true
        )
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["es"])?.isHiddenFromSearch == false)
        XCTAssertTrue(
            rootNode.descendantNodeFor(codes: ["es"])?.showsChildren == false
        )
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["es", "mad"])?.isHiddenFromSearch == false)
    }

    func testSearch3() throws {
        let nodes = dataSource.search(by: "Sweden")
        let rootNode = RootLocationNode(children: nodes)

        rootNode.countryFor(code: "se")?.forEachAncestor { location in
            XCTAssertFalse(location.isHiddenFromSearch)
        }
    }

    func testSearchWithEmptyText() throws {
        let nodes = dataSource.search(by: "")
        XCTAssertEqual(nodes, dataSource.nodes)
    }

    func testNodeByLocation() throws {
        var nodeByLocation = dataSource.node(by: .country("es"))
        var nodeByCode = dataSource.nodes.first?.descendantNodeFor(codes: ["es"])
        XCTAssertEqual(nodeByLocation, nodeByCode)

        nodeByLocation = dataSource.node(by: .city("es", "mad"))
        nodeByCode = dataSource.nodes.first?.descendantNodeFor(codes: ["es", "mad"])
        XCTAssertEqual(nodeByLocation, nodeByCode)

        nodeByLocation = dataSource.node(by: .hostname("es", "mad", "es1-wireguard"))
        nodeByCode = dataSource.nodes.first?.descendantNodeFor(codes: ["es1-wireguard"])
        XCTAssertEqual(nodeByLocation, nodeByCode)
    }
}

extension AllLocationsDataSourceTests {
    private func setUpDataSource() {
        let response = ServerRelaysResponseStubs.sampleRelays
        let relays = LocationRelays(relays: response.wireguard.relays, locations: response.locations)

        dataSource = AllLocationDataSource()
        dataSource.reload(relays)
    }
}
