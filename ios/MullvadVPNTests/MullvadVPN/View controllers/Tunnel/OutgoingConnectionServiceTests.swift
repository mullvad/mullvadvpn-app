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
        let result4 = try await outgoingConnectionService.getOutgoingIPv4ConnectionInfo()
        let result6 = try? await outgoingConnectionService.getOutgoingIPv6ConnectionInfo()
        if result4 == .mock, result6 == .mock {
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
        var ipv4Failed = false
        var ipv6Failed = false

        do {
            _ = try await outgoingConnectionService.getOutgoingIPv4ConnectionInfo()
        } catch {
            ipv4Failed = true
        }

        do {
            _ = try await outgoingConnectionService.getOutgoingIPv4ConnectionInfo()
        } catch {
            ipv6Failed = true
        }

        if ipv4Failed && ipv6Failed {
            failExpectation.fulfill()
        }
        await fulfillment(of: [failExpectation], timeout: .UnitTest.timeout)
    }
}

enum NetworkErrorStub: Error {
    case somethingWentWrong
}
