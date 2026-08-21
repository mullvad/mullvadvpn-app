// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import XCTest

@testable import MullvadMockData

final class OutgoingConnectionServiceTests: XCTestCase {
    func testSuccessGetOutgoingConnectionInfo() async throws {
        let mockOutgoingConnectionProxy = OutgoingConnectionProxyStub(
            ipV4: .mock,
            ipV6: .mock,
            error: nil
        )
        let outgoingConnectionService = OutgoingConnectionService(outgoingConnectionProxy: mockOutgoingConnectionProxy)
        let successExpectation = expectation(description: "Did receive exit IPs")
        let result = try await outgoingConnectionService.getOutgoingConnectionInfo()
        if result.ipv4 == .mock,
            result.ipv6 == .mock
        {
            successExpectation.fulfill()
        }
        await fulfillment(of: [successExpectation], timeout: .UnitTest.timeout)
    }

    func testFailureGetOutgoingConnectionInfo() async throws {
        let mockOutgoingConnectionProxy = OutgoingConnectionProxyStub(
            ipV4: .mock,
            ipV6: .mock,
            error: NetworkErrorStub.somethingWentWrong
        )
        let outgoingConnectionService = OutgoingConnectionService(outgoingConnectionProxy: mockOutgoingConnectionProxy)

        let failExpectation = expectation(description: "Did not receive exit IPs")
        do {
            _ = try await outgoingConnectionService.getOutgoingConnectionInfo()
        } catch {
            failExpectation.fulfill()
        }
        await fulfillment(of: [failExpectation], timeout: .UnitTest.timeout)
    }
}

enum NetworkErrorStub: Error {
    case somethingWentWrong
}
