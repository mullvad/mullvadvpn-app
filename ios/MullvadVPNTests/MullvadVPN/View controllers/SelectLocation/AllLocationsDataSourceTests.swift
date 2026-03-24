//
//  AllLocationsDataSourceTests.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-02-29.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import MullvadREST
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

    func testAddAutomaticLocation() throws {
        let automaticNode = dataSource.nodes.compactMap { $0.asAutomaticLocationNode }.first!

        XCTAssertTrue(automaticNode.name == "Automatic")
        XCTAssertTrue(automaticNode.code == "automatic")
        XCTAssertTrue(automaticNode.locations.isEmpty)
        XCTAssertTrue(automaticNode.locationInfo == nil)
    }

    func testSearchCity() throws {
        dataSource.search(by: "got")
        let rootNode = RootLocationNode(children: dataSource.nodes)

        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se", "got"])?.isHiddenFromSearch == false)
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se", "sto"])?.isHiddenFromSearch == true)
    }

    func testSearchShowsParentsAndChildrenIfBothMatch() throws {
        dataSource.search(by: "se")
        let rootNode = RootLocationNode(children: dataSource.nodes)

        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se"])?.isHiddenFromSearch == false)
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se", "got"])?.isHiddenFromSearch == false)
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se10-wireguard"])?.isHiddenFromSearch == false)
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se", "sto"])?.isHiddenFromSearch == false)
        XCTAssertTrue(rootNode.descendantNodeFor(codes: ["se2-wireguard"])?.isHiddenFromSearch == false)
    }

    func testSearchCityExpandsParents() throws {
        dataSource.search(by: "Sweden")
        let rootNode = RootLocationNode(children: dataSource.nodes)
        let node = rootNode.descendantNodeFor(codes: ["se"])!

        node.forEachAncestor { location in
            XCTAssertFalse(location.isHiddenFromSearch)
            XCTAssertTrue(location.showsChildren)
        }
        XCTAssertFalse(node.isHiddenFromSearch)
        XCTAssertFalse(node.showsChildren)
    }

    func testSearchCityIncludesChildren() throws {
        dataSource.search(by: "Sweden")
        let rootNode = RootLocationNode(children: dataSource.nodes)
        let node = rootNode.descendantNodeFor(codes: ["se"])!

        node.forEachDescendant { child in
            XCTAssertFalse(child.isHiddenFromSearch)
            XCTAssertFalse(child.showsChildren)
        }
        XCTAssertFalse(node.isHiddenFromSearch)
        XCTAssertFalse(node.showsChildren)
    }

    func testSearchWithEmptyText() throws {
        dataSource.search(by: "")
        dataSource.nodes.forEachNode {
            XCTAssertFalse($0.isHiddenFromSearch)
        }
    }

    func testNodeByLocation() throws {
        let rootNode = RootLocationNode(children: dataSource.nodes)

        var nodeByLocation = dataSource.node(by: .only(.init(locations: [.country("es")])))
        var nodeByCode = rootNode.descendantNodeFor(codes: ["es"])
        XCTAssertEqual(nodeByLocation, nodeByCode)

        nodeByLocation = dataSource.node(by: .only(.init(locations: [.city("es", "mad")])))
        nodeByCode = rootNode.descendantNodeFor(codes: ["es", "mad"])
        XCTAssertEqual(nodeByLocation, nodeByCode)

        nodeByLocation = dataSource.node(by: .only(.init(locations: [.hostname("es", "mad", "es1-wireguard")])))
        nodeByCode = rootNode.descendantNodeFor(codes: ["es1-wireguard"])
        XCTAssertEqual(nodeByLocation, nodeByCode)
    }

    func testConnectedNodeWithValidHostname() throws {
        let hostname = "es1-wireguard"
        let constraint = RelayConstraint<UserSelectedRelays>.only(.init(locations: [.hostname("es", "mad", hostname)]))
        let selectedRelay = SelectedRelay(
            endpoint: .init(
                socketAddress: .ipv4(.init(ip: .loopback, port: 0)),
                ipv4Gateway: .loopback,
                ipv6Gateway: .loopback,
                publicKey: Data(),
                obfuscation: .off
            ),
            hostname: hostname,
            location: .init(
                country: "",
                countryCode: "",
                city: "",
                cityCode: "",
                latitude: 0,
                longitude: 0
            ),
            features: nil
        )

        dataSource.setConnectedRelay(relayConstraint: constraint, selectedRelay: selectedRelay)
        dataSource.nodes.forEachNode { node in
            XCTAssertEqual(node.isConnected, node.name == hostname)
        }
    }

    func testConnectedNodeWithInvalidHostname() throws {
        let constraint = RelayConstraint<UserSelectedRelays>.only(
            .init(locations: [.hostname("es", "mad", "es1-wireguard")]))
        let selectedRelay = SelectedRelay(
            endpoint: .init(
                socketAddress: .ipv4(.init(ip: .loopback, port: 0)),
                ipv4Gateway: .loopback,
                ipv6Gateway: .loopback,
                publicKey: Data(),
                obfuscation: .off
            ),
            hostname: "invalid-hostname",
            location: .init(
                country: "",
                countryCode: "",
                city: "",
                cityCode: "",
                latitude: 0,
                longitude: 0
            ),
            features: nil
        )

        dataSource.setConnectedRelay(relayConstraint: constraint, selectedRelay: selectedRelay)
        dataSource.nodes.forEachNode { node in
            XCTAssertFalse(node.isConnected)
        }
    }

    func testConnectedNodeWithAutomaticLocation() throws {
        let constraint = RelayConstraint<UserSelectedRelays>.any
        let selectedRelay = SelectedRelay(
            endpoint: .init(
                socketAddress: .ipv4(.init(ip: .loopback, port: 0)),
                ipv4Gateway: .loopback,
                ipv6Gateway: .loopback,
                publicKey: Data(),
                obfuscation: .off
            ),
            hostname: "",
            location: .init(
                country: "Sweden",
                countryCode: "",
                city: "Gothenburg",
                cityCode: "",
                latitude: 0,
                longitude: 0
            ),
            features: nil
        )

        dataSource.setConnectedRelay(relayConstraint: constraint, selectedRelay: selectedRelay)

        let connectedNodes = dataSource.nodes.filter { node in
            node.isConnected
        }
        XCTAssert(connectedNodes.count == 1)

        let connectedNode = try XCTUnwrap(connectedNodes.first?.asAutomaticLocationNode)
        XCTAssertTrue(connectedNode.isConnected)
        XCTAssertEqual(connectedNode.locationInfo, ["Sweden", "Gothenburg"])
    }

    func testSetSelectedLocation() throws {
        dataSource.setSelectedNode(constraint: .only(.init(locations: [.country("es")])))

        dataSource.nodes.forEachNode { node in
            if node.locations == [.country("es")] {
                XCTAssertTrue(node.isSelected)
            } else {
                XCTAssertFalse(node.isSelected)
            }
        }

        dataSource
            .setSelectedNode(
                constraint: .only(.init(locations: [.country("invalid")]))
            )
        dataSource.nodes.forEachNode { node in
            XCTAssertFalse(node.isSelected)
        }
    }

    func testExcludeLocation() throws {
        let excludedRelays = UserSelectedRelays(locations: [.hostname("se", "sto", "se2-wireguard")])
        dataSource.setExcludedNode(constraint: .only(excludedRelays))
        let excludedNode = dataSource.node(by: .only(excludedRelays))!

        XCTAssertTrue(excludedNode.isExcluded)

        excludedNode.forEachAncestor { ancestor in
            XCTAssertFalse(ancestor.isExcluded)
        }

        let includedNode = dataSource.node(by: .only(.init(locations: [.country("es")])))!
        XCTAssertFalse(includedNode.isExcluded)
        includedNode.forEachDescendant { child in
            XCTAssertFalse(child.isExcluded)
        }
    }

    func testSinglehopDoesNotExcludeLocations() throws {
        // jp1-wireguard is the only relay in Japan. In multihop, selecting it
        // as entry correctly excludes all of Japan from exit selection.
        // But in singlehop, nothing should be excluded.
        let entryRelays = UserSelectedRelays(locations: [.hostname("jp", "tyo", "jp1-wireguard")])

        // Simulate multihop: exclusion is applied, Japan is excluded.
        dataSource.setExcludedNode(constraint: .only(entryRelays))
        let jpNode = dataSource.node(by: .only(entryRelays))!
        XCTAssertTrue(jpNode.isExcluded)

        // Simulate switching to singlehop: the view model should pass nil
        // to clear exclusions rather than passing the entry relay.
        dataSource.setExcludedNode(constraint: nil)

        // In singlehop, Japan should NOT be excluded
        XCTAssertFalse(jpNode.isExcluded)
        jpNode.forEachAncestor { ancestor in
            XCTAssertFalse(ancestor.isExcluded)
        }
    }

    func testExcludeLocationIncludesAncestors() throws {
        let excludedRelays = UserSelectedRelays(locations: [.hostname("jp", "tyo", "jp1-wireguard")])
        dataSource.setExcludedNode(constraint: .only(excludedRelays))
        let excludedNode = dataSource.node(by: .only(excludedRelays))!

        XCTAssertTrue(excludedNode.isExcluded)

        // All ancestors are exluded when single child is excluded
        excludedNode.forEachAncestor { ancestor in
            XCTAssertTrue(ancestor.isExcluded)
        }

        let includedNode = dataSource.node(by: .only(.init(locations: [.country("se")])))!
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
        dataSource.addAutomaticLocationNode()
    }
}
