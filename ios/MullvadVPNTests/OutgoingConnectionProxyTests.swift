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

    private let encoder: JSONEncoder = REST.Coding.makeJSONEncoder()

    override func setUp() {
        outgoingConnectionProxy = OutgoingConnectionProxy(urlSession: URLSession.mock)
        // swiftlint:disable force_try
        mockIPV4ConnectionData = try! encoder.encode(OutgoingConnectionProxy.IPV4ConnectionData.mock)
        mockIPV6ConnectionData = try! encoder.encode(OutgoingConnectionProxy.IPV6ConnectionData.mock)
        // swiftlint:enable force_try
    }

    override func tearDown() {
        outgoingConnectionProxy = nil
        mockIPV4ConnectionData.removeAll()
        mockIPV6ConnectionData.removeAll()
    }

    func testNoInternetConnection() async throws {
        let failureExpectation = expectation(description: "should fail")
        let successExpectation = expectation(description: "should not succeed")
        successExpectation.isInverted = true

        var thrownError: Error?
        let errorHandler = { thrownError = $0 }
        let error = URLError(URLError.notConnectedToInternet)

        MockURLProtocol.error = error
        MockURLProtocol.requestHandler = nil

        do {
            _ = try await outgoingConnectionProxy.getIPV4(retryStrategy: .noRetry)
            successExpectation.fulfill()
        } catch {
            errorHandler(error)
            failureExpectation.fulfill()
        }

        await fulfillment(of: [successExpectation, failureExpectation], timeout: 1.0)
        XCTAssertEqual(error.localizedDescription, thrownError?.localizedDescription)
    }

    func testSuccessGettingIPV4() async throws {
        let successExpectation = expectation(description: "should succeed")
        let failureExpectation = expectation(description: "should not fail")
        failureExpectation.isInverted = true

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

        if result.ip == OutgoingConnectionProxy.IPV4ConnectionData.mock.ip {
            successExpectation.fulfill()
        } else {
            failureExpectation.fulfill()
        }
        await fulfillment(of: [successExpectation, failureExpectation], timeout: 1.0)
    }

    func testFailureGettingIPV4() async throws {
        let failureExpectation = expectation(description: "should fail")
        let successExpectation = expectation(description: "should not succeed")
        successExpectation.isInverted = true

        var thrownError: Error?
        let errorHandler = { thrownError = $0 }

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

        do {
            _ = try await outgoingConnectionProxy.getIPV4(retryStrategy: .default)
            successExpectation.fulfill()
        } catch {
            errorHandler(error)
            failureExpectation.fulfill()
        }

        await fulfillment(of: [successExpectation, failureExpectation], timeout: 1.0)
        if let restError = thrownError as? REST.Error,
           case let REST.Error.unhandledResponse(statusCode, _) = restError {
            XCTAssertEqual(503, statusCode)
        } else {
            XCTFail("Unexpected Error!")
        }
    }

    func testSuccessGettingIPV6() async throws {
        let successExpectation = expectation(description: "should succeed")
        let failureExpectation = expectation(description: "should not fail")
        failureExpectation.isInverted = true

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

        if result.ip == OutgoingConnectionProxy.IPV6ConnectionData.mock.ip {
            successExpectation.fulfill()
        } else {
            failureExpectation.fulfill()
        }
        await fulfillment(of: [successExpectation, failureExpectation], timeout: 1.0)
    }

    func testFailureGettingIPV6() async throws {
        let failureExpectation = expectation(description: "should fail")
        let successExpectation = expectation(description: "should not succeed")
        successExpectation.isInverted = true

        var thrownError: Error?
        let errorHandler = { thrownError = $0 }

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

        do {
            _ = try await outgoingConnectionProxy.getIPV6(retryStrategy: .noRetry)
            successExpectation.fulfill()
        } catch {
            errorHandler(error)
            failureExpectation.fulfill()
        }

        await fulfillment(of: [successExpectation, failureExpectation], timeout: 1.0)
        if let restError = thrownError as? REST.Error,
           case let REST.Error.unhandledResponse(statusCode, _) = restError {
            XCTAssertEqual(404, statusCode)
        } else {
            XCTFail("Unexpected Error!")
        }
    }
}
