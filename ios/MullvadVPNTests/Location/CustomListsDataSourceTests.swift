//
//  CustomListsDataSourceTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-02-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadSettings
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

        let netflixNode = try XCTUnwrap(nodes.first(where: { $0.nodeName == "Netflix" }))
        XCTAssertNotNil(netflixNode.childNodeFor(nodeCode: "netflix-es1-wireguard"))
        XCTAssertNotNil(netflixNode.childNodeFor(nodeCode: "netflix-se"))
        XCTAssertNotNil(netflixNode.childNodeFor(nodeCode: "netflix-dal"))

        let youtubeNode = try XCTUnwrap(nodes.first(where: { $0.nodeName == "Youtube" }))
        XCTAssertNotNil(youtubeNode.childNodeFor(nodeCode: "youtube-se2-wireguard"))
        XCTAssertNotNil(youtubeNode.childNodeFor(nodeCode: "youtube-dal"))
    }

    func testSearch() throws {
        let nodes = dataSource.search(by: "got")
        let rootNode = RootLocationNode(children: nodes)

        XCTAssertTrue(rootNode.childNodeFor(nodeCode: "netflix-got")?.isHiddenFromSearch == false)
        XCTAssertTrue(rootNode.childNodeFor(nodeCode: "netflix-sto")?.isHiddenFromSearch == true)
    }

    func testSearchWithEmptyText() throws {
        let nodes = dataSource.search(by: "")
        XCTAssertEqual(nodes, dataSource.nodes)
    }

    func testSearchYieldsNoListNodes() throws {
        let nodes = dataSource.search(by: "net")
        XCTAssertFalse(nodes.contains(where: { $0.nodeName == "Netflix" }))
    }

    func testNodeByLocations() throws {
        let nodeByLocations = dataSource.node(by: [.hostname("es", "mad", "es1-wireguard")], for: customLists.first!)
        let nodeByCode = dataSource.nodes.first?.childNodeFor(nodeCode: "netflix-es1-wireguard")

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
        let relays = response.wireguard.relays

        let dataSource = AllLocationDataSource()
        dataSource.reload(response, relays: relays)

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
