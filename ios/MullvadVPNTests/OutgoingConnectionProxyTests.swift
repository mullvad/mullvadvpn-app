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
    private var outgoingConnectionProxy: OutgoingConnectionProxy!
    private var mockIPV6ConnectionData: Data!
    private var mockIPV4ConnectionData: Data!

    private let encoder = JSONEncoder()

    override func setUpWithError() throws {
        outgoingConnectionProxy = OutgoingConnectionProxy(urlSession: .mock)
        mockIPV4ConnectionData = try encoder.encode(IPV4ConnectionData.mock)
        mockIPV6ConnectionData = try encoder.encode(IPV6ConnectionData.mock)
    }

    override func tearDownWithError() throws {
        outgoingConnectionProxy = nil
        mockIPV4ConnectionData.removeAll()
        mockIPV6ConnectionData.removeAll()
    }

    func testNoInternetConnection() async throws {
        let noIPv4Expectation = expectation(description: "Did not receive IPv4")
        let error = URLError(URLError.notConnectedToInternet)

        MockURLProtocol.error = error
        MockURLProtocol.requestHandler = nil

        await XCTAssertThrowsErrorAsync(try await outgoingConnectionProxy.getIPV4(retryStrategy: .noRetry)) { error in
            noIPv4Expectation.fulfill()
            XCTAssertEqual((error as? URLError)?.code, .notConnectedToInternet)
        }
        await fulfillment(of: [noIPv4Expectation], timeout: 1)
    }

    func testSuccessGettingIPV4() async throws {
        let iPv4Expectation = expectation(description: "Did receive IPv4")

        MockURLProtocol.error = nil
        MockURLProtocol.requestHandler = { _ in
            let response = HTTPURLResponse(
                url: URL(string: "https://ipv4.am.i.mullvad.net/json")!,
                statusCode: 200,
                httpVersion: nil,
                headerFields: ["Content-Type": "application/json"]
            )!
            return (response, self.mockIPV4ConnectionData)
        }

        let result = try await outgoingConnectionProxy.getIPV4(retryStrategy: .noRetry)

        if result.ip == IPV4ConnectionData.mock.ip {
            iPv4Expectation.fulfill()
        }
        await fulfillment(of: [iPv4Expectation], timeout: 1)
    }

    func testFailureGettingIPV4() async throws {
        let noIPv4Expectation = expectation(description: "Did not receive IPv4")

        MockURLProtocol.error = nil
        MockURLProtocol.requestHandler = { _ in
            let response = HTTPURLResponse(
                url: URL(string: "https://ipv4.am.i.mullvad.net/json")!,
                statusCode: 503,
                httpVersion: nil,
                headerFields: ["Content-Type": "application/json"]
            )!
            return (response, Data())
        }

        await XCTAssertThrowsErrorAsync(try await outgoingConnectionProxy.getIPV4(retryStrategy: .noRetry)) { _ in
            noIPv4Expectation.fulfill()
        }
        await fulfillment(of: [noIPv4Expectation], timeout: 1)
    }

    func testSuccessGettingIPV6() async throws {
        let ipv6Expectation = expectation(description: "Did receive IPv6")

        MockURLProtocol.error = nil
        MockURLProtocol.requestHandler = { _ in
            let response = HTTPURLResponse(
                url: URL(string: "https://ipv6.am.i.mullvad.net/json")!,
                statusCode: 200,
                httpVersion: nil,
                headerFields: ["Content-Type": "application/json"]
            )!
            return (response, self.mockIPV6ConnectionData)
        }

        let result = try await outgoingConnectionProxy.getIPV6(retryStrategy: .noRetry)

        if result.ip == IPV6ConnectionData.mock.ip {
            ipv6Expectation.fulfill()
        }
        await fulfillment(of: [ipv6Expectation], timeout: 1.0)
    }

    func testFailureGettingIPV6() async throws {
        let noIPv6Expectation = expectation(description: "Did not receive IPv6")

        MockURLProtocol.error = nil
        MockURLProtocol.requestHandler = { _ in
            let response = HTTPURLResponse(
                url: URL(string: "https://ipv6.am.i.mullvad.net/json")!,
                statusCode: 404,
                httpVersion: nil,
                headerFields: ["Content-Type": "application/json"]
            )!
            return (response, Data())
        }

        await XCTAssertThrowsErrorAsync(try await outgoingConnectionProxy.getIPV6(retryStrategy: .noRetry)) { _ in
            noIPv6Expectation.fulfill()
        }
        await fulfillment(of: [noIPv6Expectation], timeout: 1)
    }
}
