//
//  OutgoingConnectionProxyTests.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2023-10-25.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//
import MullvadREST
import XCTest

final class OutgoingConnectionProxyTests: XCTestCase {
    private var mockIPV6ConnectionData: Data!
    private var mockIPV4ConnectionData: Data!

    private let encoder = JSONEncoder()

    override func setUpWithError() throws {
        mockIPV4ConnectionData = try encoder.encode(IPV4ConnectionData.mock)
        mockIPV6ConnectionData = try encoder.encode(IPV6ConnectionData.mock)
    }

    override func tearDownWithError() throws {
        mockIPV4ConnectionData.removeAll()
        mockIPV6ConnectionData.removeAll()
    }

    func testSuccessGettingIPV4() async throws {
        let iPv4Expectation = expectation(description: "Did receive IPv4")

        let outgoingConnectionProxy = OutgoingConnectionProxy(urlSession: URLSessionStub(
            response: (mockIPV4ConnectionData, createHTTPURLResponse(ip: .v4, statusCode: 200))
        ))

        let result = try await outgoingConnectionProxy.getIPV4(retryStrategy: .noRetry)

        if result.ip == IPV4ConnectionData.mock.ip {
            iPv4Expectation.fulfill()
        }
        await fulfillment(of: [iPv4Expectation], timeout: 1)
    }

    func testFailureGettingIPV4() async throws {
        let noIPv4Expectation = expectation(description: "Did not receive IPv4")

        let outgoingConnectionProxy = OutgoingConnectionProxy(urlSession: URLSessionStub(
            response: (Data(), createHTTPURLResponse(ip: .v4, statusCode: 503))
        ))

        await XCTAssertThrowsErrorAsync(try await outgoingConnectionProxy.getIPV4(retryStrategy: .noRetry)) { _ in
            noIPv4Expectation.fulfill()
        }
        await fulfillment(of: [noIPv4Expectation], timeout: 1)
    }

    func testSuccessGettingIPV6() async throws {
        let ipv6Expectation = expectation(description: "Did receive IPv6")

        let outgoingConnectionProxy = OutgoingConnectionProxy(urlSession: URLSessionStub(
            response: (mockIPV6ConnectionData, createHTTPURLResponse(ip: .v6, statusCode: 200))
        ))

        let result = try await outgoingConnectionProxy.getIPV6(retryStrategy: .noRetry)

        if result.ip == IPV6ConnectionData.mock.ip {
            ipv6Expectation.fulfill()
        }
        await fulfillment(of: [ipv6Expectation], timeout: 1.0)
    }

    func testFailureGettingIPV6() async throws {
        let noIPv6Expectation = expectation(description: "Did not receive IPv6")

        let outgoingConnectionProxy = OutgoingConnectionProxy(urlSession: URLSessionStub(
            response: (mockIPV6ConnectionData, createHTTPURLResponse(ip: .v6, statusCode: 404))
        ))

        await XCTAssertThrowsErrorAsync(try await outgoingConnectionProxy.getIPV6(retryStrategy: .noRetry)) { _ in
            noIPv6Expectation.fulfill()
        }
        await fulfillment(of: [noIPv6Expectation], timeout: 1)
    }
}

extension OutgoingConnectionProxyTests {
    private func createHTTPURLResponse(ip: OutgoingConnectionProxy.ExitIPVersion, statusCode: Int) -> HTTPURLResponse {
        return HTTPURLResponse(
            url: URL(string: "https://\(ip.host)/json")!,
            statusCode: statusCode,
            httpVersion: nil,
            headerFields: ["Content-Type": "application/json"]
        )!
    }
}
