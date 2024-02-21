//
//  AllLocationsDataSourceTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-02-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
import XCTest

class AllLocationsDataSourceTests: XCTestCase {
    var allLocationNodes = [LocationNode]()
    var dataSource: AllLocationDataSource!

    override func setUp() async throws {
        setUpDataSource()
    }

    func testNodeTree() throws {
        let rootNode = RootLocationNode(children: dataSource.nodes)

        // Testing a selection.
        XCTAssertNotNil(rootNode.childNodeFor(nodeCode: "se"))
        XCTAssertNotNil(rootNode.childNodeFor(nodeCode: "dal"))
        XCTAssertNotNil(rootNode.childNodeFor(nodeCode: "es1-wireguard"))
        XCTAssertNotNil(rootNode.childNodeFor(nodeCode: "se2-wireguard"))
    }

    func testSearch() throws {
        let nodes = dataSource.search(by: "got")
        let rootNode = RootLocationNode(children: nodes)

        XCTAssertTrue(rootNode.childNodeFor(nodeCode: "got")?.isHiddenFromSearch == false)
        XCTAssertTrue(rootNode.childNodeFor(nodeCode: "sto")?.isHiddenFromSearch == true)
    }

    func testSearchWithEmptyText() throws {
        let nodes = dataSource.search(by: "")
        XCTAssertEqual(nodes, dataSource.nodes)
    }

    func testNodeByLocation() throws {
        let nodeByLocation = dataSource.node(by: .hostname("es", "mad", "es1-wireguard"))
        let nodeByCode = dataSource.nodes.first?.childNodeFor(nodeCode: "es1-wireguard")

        XCTAssertEqual(nodeByLocation, nodeByCode)
    }
}

extension AllLocationsDataSourceTests {
    private func setUpDataSource() {
        let response = ServerRelaysResponseStubs.sampleRelays
        let relays = response.wireguard.relays

        dataSource = AllLocationDataSource()
        dataSource.reload(response, relays: relays)
    }
}
