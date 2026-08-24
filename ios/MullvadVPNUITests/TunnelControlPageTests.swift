// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import XCTest

class TunnelControlPageTests: XCTestCase {
    func testConnectionDetailValueStripsInPrefix() {
        let result = TunnelControlPage.connectionDetailValue(from: "In 85.203.53.104:56678 UDP")
        XCTAssertEqual(result, "85.203.53.104:56678 UDP")
    }

    func testConnectionDetailValueStripsOutIPv4Prefix() {
        let result = TunnelControlPage.connectionDetailValue(from: "Out IPv4 192.168.1.1")
        XCTAssertEqual(result, "192.168.1.1")
    }

    func testConnectionDetailValueStripsOutIPv6Prefix() {
        let result = TunnelControlPage.connectionDetailValue(from: "Out IPv6 2001:db8::1")
        XCTAssertEqual(result, "2001:db8::1")
    }

    func testConnectionDetailValueReturnsLabelWhenNoAddressFound() {
        let result = TunnelControlPage.connectionDetailValue(from: "No address")
        XCTAssertEqual(result, "No address")
    }

    func testConnectionDetailValueHandlesIPAddressOnly() {
        let result = TunnelControlPage.connectionDetailValue(from: "In 85.203.53.104")
        XCTAssertEqual(result, "85.203.53.104")
    }
}
