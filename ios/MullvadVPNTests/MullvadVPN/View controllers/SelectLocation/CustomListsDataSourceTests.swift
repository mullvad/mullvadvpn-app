//
//  CustomListsDataSourceTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-02-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

class CustomListsDataSourceTests: XCTestCase {
    var allLocationNodes = [LocationNode]()
    var dataSource: CustomListsDataSource!

    override func setUp() async throws {
        createAllLocationNodes()
        setUpDataSource()
    }

    func testNodeTree() throws {
        let nodes = dataSource.nodes

        let netflixNode = try XCTUnwrap(nodes.first(where: { $0.name == "Netflix" }))
        XCTAssertNotNil(netflixNode.descendantNodeFor(codes: ["Netflix", "es1-wireguard"]))
        XCTAssertNotNil(netflixNode.descendantNodeFor(codes: ["Netflix", "se"]))
        XCTAssertNotNil(netflixNode.descendantNodeFor(codes: ["Netflix", "us", "dal"]))

        let youtubeNode = try XCTUnwrap(nodes.first(where: { $0.name == "Youtube" }))
        XCTAssertNotNil(youtubeNode.descendantNodeFor(codes: ["Youtube", "se2-wireguard"]))
        XCTAssertNotNil(youtubeNode.descendantNodeFor(codes: ["Youtube", "us", "dal"]))
    }

    func testParents() throws {
        let listNode = try XCTUnwrap(dataSource.nodes.first(where: { $0.name == "Netflix" }))
        let countryNode = try XCTUnwrap(listNode.descendantNodeFor(codes: ["Netflix", "se"]))
        let cityNode = try XCTUnwrap(listNode.descendantNodeFor(codes: ["Netflix", "se", "got"]))
        let hostNode = try XCTUnwrap(listNode.descendantNodeFor(codes: ["Netflix-se10-wireguard"]))

        XCTAssertNil(listNode.parent)
        XCTAssertEqual(countryNode.parent, listNode)
        XCTAssertEqual(cityNode.parent, countryNode)
        XCTAssertEqual(hostNode.parent, cityNode)
    }

    func testSearch() throws {
        let nodes = dataSource.search(by: "got")
        let rootNode = RootLocationNode(children: nodes)

        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["Netflix", "se", "got"])?.isHiddenFromSearch == false)
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["Netflix", "se", "sto"])?.isHiddenFromSearch == true)
    }

    func testSearchWithEmptyText() throws {
        let nodes = dataSource.search(by: "")
        XCTAssertEqual(nodes, dataSource.nodes)
    }

    func testSearchYieldsNoListNodes() throws {
        let nodes = dataSource.search(by: "net")
        XCTAssertFalse(nodes.contains(where: { $0.name == "Netflix" }))
    }

    func testNodeByLocations() throws {
        let relays = UserSelectedRelays(locations: [.hostname("es", "mad", "es1-wireguard")], customListSelection: nil)

        let nodeByLocations = dataSource.node(by: relays, for: customLists.first!)
        let nodeByCode = dataSource.nodes.first?.descendantNodeFor(codes: ["Netflix", "es1-wireguard"])

        XCTAssertEqual(nodeByLocations, nodeByCode)
    }
}

extension CustomListsDataSourceTests {
    private func setUpDataSource() {
        dataSource = CustomListsDataSource(repository: CustomListsRepositoryStub(customLists: customLists))
        dataSource.reload(allLocationNodes: allLocationNodes)
    }

    private func createAllLocationNodes() {
        let response = ServerRelaysResponseStubs.sampleRelays
        let relays = LocationRelays(relays: response.wireguard.relays, locations: response.locations)

        let dataSource = AllLocationDataSource()
        dataSource.reload(relays)

        allLocationNodes = dataSource.nodes
    }

    var customLists: [CustomList] {
        [
            CustomList(name: "Netflix", locations: [
                .hostname("es", "mad", "es1-wireguard"),
                .country("se"),
                .city("us", "dal"),
            ]),
            CustomList(name: "Youtube", locations: [
                .hostname("se", "sto", "se2-wireguard"),
                .city("us", "dal"),
            ]),
        ]
    }
}
