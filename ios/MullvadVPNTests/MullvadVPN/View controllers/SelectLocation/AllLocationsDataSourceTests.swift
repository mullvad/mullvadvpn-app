//
//  AllLocationsDataSourceTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-02-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import MullvadTypes
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
        dataSource.search(by: "got")
        let rootNode = RootLocationNode(children: dataSource.nodes)

        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se", "got"])?.isHiddenFromSearch == false)
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se", "sto"])?.isHiddenFromSearch == true)
    }

    func testSearch2() throws {
        dataSource.search(by: "s")
        let rootNode = RootLocationNode(children: dataSource.nodes)

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
            rootNode.descendantNodeFor(codes: ["es"])?.showsChildren == true
        )
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["es", "mad"])?.isHiddenFromSearch == false)
        XCTAssertTrue(
            rootNode
                .descendantNodeFor(codes: ["es", "mad"])?.showsChildren == true
        )
    }

    func testSearch3() throws {
        dataSource.search(by: "Sweden")
        let rootNode = RootLocationNode(children: dataSource.nodes)

        rootNode.countryFor(code: "se")?.forEachAncestor { location in
            XCTAssertFalse(location.isHiddenFromSearch)
        }
    }

    func testSearchWithEmptyText() throws {
        dataSource.search(by: "")
        dataSource.nodes.forEachNode {
            XCTAssertFalse($0.isHiddenFromSearch)
        }
    }

    func testNodeByLocation() throws {
        var nodeByLocation = dataSource.node(by: .init(locations: [.country("es")]))
        var nodeByCode = dataSource.nodes.first?.descendantNodeFor(codes: ["es"])
        XCTAssertEqual(nodeByLocation, nodeByCode)

        nodeByLocation = dataSource.node(by: .init(locations: [.city("es", "mad")]))
        nodeByCode = dataSource.nodes.first?.descendantNodeFor(codes: ["es", "mad"])
        XCTAssertEqual(nodeByLocation, nodeByCode)

        nodeByLocation = dataSource.node(by: .init(locations: [.hostname("es", "mad", "es1-wireguard")]))
        nodeByCode = dataSource.nodes.first?.descendantNodeFor(codes: ["es1-wireguard"])
        XCTAssertEqual(nodeByLocation, nodeByCode)
    }

    func testConnectedNode() throws {
        let hostname = "es1-wireguard"
        dataSource.setConnectedRelay(hostname: hostname)
        dataSource.nodes.forEachNode { node in
            XCTAssertEqual(node.isConnected, node.name == hostname)
        }

        dataSource.setConnectedRelay(hostname: "invalid-hostname")
        dataSource.nodes.forEachNode { node in
            XCTAssertFalse(node.isConnected)
        }
    }

    func testSetSelectedLocation() throws {
        dataSource.setSelectedNode(selectedRelays: .init(locations: [.country("es")]))

        dataSource.nodes.forEachNode { node in
            if node.locations == [.country("es")] {
                XCTAssertTrue(node.isSelected)
            } else {
                XCTAssertFalse(node.isSelected)
            }
        }

        dataSource
            .setSelectedNode(
                selectedRelays: .init(locations: [.country("invalid")])
            )
        dataSource.nodes.forEachNode { node in
            XCTAssertFalse(node.isSelected)
        }
    }

    func testDoNotSetSelectedCustomListLocation() throws {
        let selectedRelays: UserSelectedRelays = .init(
            locations: [
                .country("es")
            ],
            customListSelection: .init(listId: .init(), isList: false)
        )

        dataSource.setSelectedNode(selectedRelays: selectedRelays)

        dataSource.nodes.forEachNode { node in
            XCTAssertFalse(node.isSelected)
        }
    }

    func testExcludeLocation() throws {
        let excludedRelays = UserSelectedRelays(locations: [.hostname("se", "sto", "se2-wireguard")])
        dataSource.setExcludedNode(excludedRelays: excludedRelays)
        let excludedNode = dataSource.node(by: excludedRelays)!

        XCTAssertTrue(excludedNode.isExcluded)

        excludedNode.forEachAncestor { ancestor in
            XCTAssertFalse(ancestor.isExcluded)
        }

        let includedNode = dataSource.node(by: .init(locations: [.country("es")]))!
        XCTAssertFalse(includedNode.isExcluded)
        includedNode.forEachDescendant { child in
            XCTAssertFalse(child.isExcluded)
        }
    }

    func testExcludeLocationIncludesAncestors() throws {
        let excludedRelays = UserSelectedRelays(locations: [.hostname("es", "mad", "es1-wireguard")])
        dataSource.setExcludedNode(excludedRelays: excludedRelays)
        let excludedNode = dataSource.node(by: excludedRelays)!

        XCTAssertTrue(excludedNode.isExcluded)

        // All ancestors are exluded when single child is excluded
        excludedNode.forEachAncestor { ancestor in
            XCTAssertTrue(ancestor.isExcluded)
        }

        let includedNode = dataSource.node(by: .init(locations: [.country("se")]))!
        XCTAssertFalse(includedNode.isExcluded)
        includedNode.forEachDescendant { child in
            XCTAssertFalse(child.isExcluded)
        }
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
