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
    private var mockOutgoingConnectionProxy: OutgoingConnectionProxyStub!
    private var outgoingConnectionService: OutgoingConnectionService!

    override func setUp() {
        mockOutgoingConnectionProxy = OutgoingConnectionProxyStub(
            ipV4: .mock,
            ipV6: .mock,
            error: MockNetworkError.somethingWentWrong
        )
        outgoingConnectionService = OutgoingConnectionService(outgoingConnectionProxy: mockOutgoingConnectionProxy)
    }

    override func tearDown() {
        mockOutgoingConnectionProxy = nil
        outgoingConnectionService = nil
    }

    func testSuccessGetOutgoingConnectionInfo() async throws {
        let successExpectation = expectation(description: "should succeed")
        OutgoingConnectionProxyStub.hasError = false
        let result = try await outgoingConnectionService.getOutgoingConnectionInfo()
        if result.ipv4 == .mock,
           result.ipv6 == .mock {
            successExpectation.fulfill()
        }
        await fulfillment(of: [successExpectation], timeout: 1.0)
    }

    func testFailureGetOutgoingConnectionInfo() async throws {
        let failExpectation = expectation(description: "should fail")
        OutgoingConnectionProxyStub.hasError = true
        do {
            _ = try await outgoingConnectionService.getOutgoingConnectionInfo()
        } catch {
            failExpectation.fulfill()
        }
        await fulfillment(of: [failExpectation], timeout: 1.0)
    }
}

enum MockNetworkError: Error {
    case somethingWentWrong
}
