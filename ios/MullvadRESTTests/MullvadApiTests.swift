//
//  MullvadApiTests.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-02-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import MullvadRustRuntime
import MullvadTypes
import Network
import XCTest

/// This tests main purpose is to test the functionallity of the FFI rather than every function of the proxy itself.
/// It makes sure the response and errors are parsed correctly.

class MullvadApiTests: XCTestCase {
    let encoder = JSONEncoder()

    func makeApiProxy(port: UInt16) throws -> APIQuerying {
        let context = try MullvadApiContext(host: "localhost", address: .ipv4(
            .init(ip: IPv4Address.loopback, port: port)
        ), disable_tls: true)
        let proxy = REST.MullvadAPIProxy(
            transportProvider: APITransportProvider(
                requestFactory: .init(apiContext: context)
            ),
            dispatchQueue: .main,
            responseDecoder: REST.Coding.makeJSONDecoder()
        )

        return proxy
    }

    func testSuccessfulResponse() async throws {
        let expectedEndpoints: [AnyIPEndpoint] = [AnyIPEndpoint(string: "12.34.56.78:80")!]

        let bodyData = try encoder.encode(expectedEndpoints)
        let body = String(data: bodyData, encoding: .utf8)!
        let responseCode: UInt = 200
        let mock = mullvad_api_mock_get(
            "/app/v1/api-addrs",
            UInt(responseCode),
            body
        )
        defer { mullvad_api_mock_drop(mock) }
        let apiProxy = try makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { continuation in
            _ = apiProxy
                .getAddressList(
                    retryStrategy: .noRetry
                ) { result in
                    continuation.resume(returning: result)
                }
        }

        guard let receivedEndpoints = result.value else {
            XCTFail(result.error!.localizedDescription)
            return
        }

        XCTAssertEqual(receivedEndpoints, expectedEndpoints)
    }

    func testHTTPError() async throws {
        let expectedResponseCode = 500
        let mock = mullvad_api_mock_get(
            "/app/v1/api-addrs",
            UInt(expectedResponseCode),
            ""
        )
        defer { mullvad_api_mock_drop(mock) }
        let apiProxy = try makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { continuation in
            _ = apiProxy
                .getAddressList(
                    retryStrategy: .noRetry
                ) { result in
                    continuation.resume(returning: result)
                }
        }

        let error = try XCTUnwrap(result.error as? REST.Error)

        switch error {
        case let .unhandledResponse(responseCode, _):
            XCTAssertEqual(responseCode, expectedResponseCode)
        default:
            XCTFail("GetAddressList failed with the wrong error: \(error)")
        }
    }

    func testInvalidBody() async throws {
        let expectedResponseCode = 200
        let mock = mullvad_api_mock_get(
            "/app/v1/api-addrs",
            UInt(expectedResponseCode),
            "This is an invalid JSON"
        )
        defer { mullvad_api_mock_drop(mock) }
        let apiProxy = try makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { continuation in
            _ = apiProxy
                .getAddressList(
                    retryStrategy: .noRetry
                ) { result in
                    continuation.resume(returning: result)
                }
        }

        let error = try XCTUnwrap(result.error as? REST.Error)

        switch error {
        case let .unhandledResponse(_, response):
            XCTAssertEqual(response?.code, REST.ServerResponseCode.parsingError)
        default:
            XCTFail("GetAddressList failed with the wrong error: \(error)")
        }
    }

    func testCustomErrorCode() async throws {
        let expectedResponseCode = 400
        let expectedErrorCode = 123
        let mock = mullvad_api_mock_get(
            "/app/v1/api-addrs",
            UInt(expectedResponseCode),
            """
            {"code": "\(expectedErrorCode)",
            "error": "A magical error occured"
            }
            """
        )
        defer { mullvad_api_mock_drop(mock) }
        let apiProxy = try makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { continuation in
            _ = apiProxy
                .getAddressList(
                    retryStrategy: .noRetry
                ) { result in
                    continuation.resume(returning: result)
                }
        }

        let error = try XCTUnwrap(result.error as? REST.Error)

        switch error {
        case let .unhandledResponse(responseCode, response):
            XCTAssertEqual(responseCode, expectedResponseCode)
            guard let response else {
                XCTFail("Expected error response object, but got nil")
                return
            }
            XCTAssertEqual(response.code.rawValue, String(expectedErrorCode))
        default:
            XCTFail("GetAddressList failed with the wrong error: \(error)")
        }
    }

    func testSuccessfulPostRequest() async throws {
        let skipReason = """
        Problem report is not yet using mullvad api. This test should be enabled once it does.
        """
        try XCTSkipIf(true, skipReason)
        let matchBody = REST.ProblemReportRequest(
            address: "test@email.com",
            message: "This test should succeed",
            log: "A long log string",
            metadata: [:]
        )
        let matchBodyString = String(data: try encoder.encode(matchBody), encoding: .utf8)!
        let expectedResponseCode: UInt = 204
        let mock = mullvad_api_mock_post(
            "/app/v1/problem-report",
            UInt(expectedResponseCode),
            matchBodyString
        )
        defer { mullvad_api_mock_drop(mock) }
        let apiProxy = try makeApiProxy(port: mock.port)

        let result: Result<Void, Error> = await withCheckedContinuation { continuation in
            _ = apiProxy
                .sendProblemReport(
                    matchBody,
                    retryStrategy:
                    .noRetry
                ) { result in
                    continuation.resume(returning: result)
                }
        }

        XCTAssertNil(result.error)
    }
}
