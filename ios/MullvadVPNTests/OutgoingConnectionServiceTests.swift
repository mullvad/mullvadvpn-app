//
//  OutgoingConnectionServiceTests.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2023-11-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

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
           result.ipv6 == .mock {
            successExpectation.fulfill()
        }
        await fulfillment(of: [successExpectation], timeout: 1.0)
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
        await fulfillment(of: [failExpectation], timeout: 1.0)
    }
}

enum NetworkErrorStub: Error {
    case somethingWentWrong
}
