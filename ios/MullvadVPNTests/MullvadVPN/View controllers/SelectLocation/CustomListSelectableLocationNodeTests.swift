// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Testing

struct CustomListSelectableLocationNodeTests {

    @Test func testAllNodesAreUnselected() async throws {
        let tree = buildNodeTree()

        verifySelectedNodes(tree, expectedCodes: [])
    }

    @Test func testSelectOneRelay() async throws {
        let tree = buildNodeTree()

        tree.selectNode("got-1")

        verifySelectedNodes(tree, expectedCodes: ["got-1"])
    }

    @Test func testSelectAllRelaysDoesNotSelectsParent() async throws {
        let tree = buildNodeTree()

        tree.selectNode("got-1")
        tree.selectNode("got-2")

        verifySelectedNodes(tree, expectedCodes: ["got-1", "got-2"])
    }

    @Test func testUnselectRelaysUnselectsParent() async throws {
        let tree = buildNodeTree()

        tree.selectNode("got")

        verifySelectedNodes(tree, expectedCodes: ["got-1", "got-2", "got"])

        tree.unselectNode("got-2")

        verifySelectedNodes(tree, expectedCodes: ["got-1"])
    }

    @Test func testUnselectCountryUnselectsAllChildren() async throws {
        let tree = buildNodeTree()

        tree.selectNodes(["se"])

        verifySelectedNodes(tree, expectedCodes: ["got-1", "got-2", "sth-1", "sth-2", "got", "sth", "se"])

        tree.unselectNode("se")

        verifySelectedNodes(tree, expectedCodes: [])
    }

    func verifySelectedNodes(
        _ node: CustomListSelectableLocationNode, expectedCodes: [String],
        sourceLocation: SourceLocation = #_sourceLocation
    ) {

        let selectedNodes = getAllSelectedNodes(node)
        #expect(
            selectedNodes.count == expectedCodes.count,
            sourceLocation: sourceLocation
        )
        let selectedCodes = selectedNodes.map(\.code).sorted()
        let expectedCodes = expectedCodes.sorted()
        #expect(
            selectedCodes == expectedCodes,
            sourceLocation: sourceLocation
        )
    }

    func getAllSelectedNodes(_ node: CustomListSelectableLocationNode) -> [CustomListSelectableLocationNode] {
        node.flattened
            .compactMap { $0.asCustomListSelectableLocationNode }
            .filter { $0.isCustomListSelected }
    }

    func buildNodeTree() -> CustomListSelectableLocationNode {
        let got_1 = LocationNode(name: "got-1", code: "got-1")
        let got_2 = LocationNode(name: "got-2", code: "got-2")

        let sth_1 = LocationNode(name: "sth-1", code: "sth-1")
        let sth_2 = LocationNode(name: "sth-2", code: "sth-2")

        let got = LocationNode(
            name: "Gothenburg",
            code: "got",
            children: [got_1, got_2]
        )
        let sth = LocationNode(
            name: "Stockholm",
            code: "sth",
            children: [sth_1, sth_2]
        )

        let sweden = LocationNode(
            name: "Sweden",
            code: "se",
            children: [got, sth]
        )
        let norway = LocationNode(name: "Norway", code: "no")

        return CustomListSelectableLocationNode(
            fromLocationNode: RootLocationNode(children: [sweden, norway])
        )
    }
}

extension CustomListSelectableLocationNode {
    func selectNode(_ code: String) {
        descendantNode(
            for: [code]
        )?.asCustomListSelectableLocationNode?.isCustomListSelected = true
    }

    func selectNodes(_ codes: [String]) {
        codes.forEach {
            selectNode($0)
        }
    }

    func unselectNode(_ code: String) {
        descendantNode(
            for: [code]
        )?.asCustomListSelectableLocationNode?.isCustomListSelected = false
    }
}
