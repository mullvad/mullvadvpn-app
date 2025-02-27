//
//  MullvadApiTests.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-02-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
@testable import MullvadREST
import MullvadRustRuntime
import MullvadTypes
import XCTest

/// This tests main purpose is to test the functionallity of the FFI rather than every function of the proxy itself.
/// It makes sure the response and errors are parsed correctly.
///  TODO: Once a POST request is implemented, a test should be added to ensure the rust side receives the correct body.
class MullvadApiTests: XCTestCase {
    let encoder = JSONEncoder()

    func makeApiProxy(port: UInt16) -> APIQuerying {
        let addressCache = REST.AddressCache(canWriteToCache: false, fileCache: MemoryCache())
        // This transport provider will never be used in these tests.
        let transportProvider = REST.AnyTransportProvider {
            AnyTransport {
                Response(delay: 1, statusCode: 500, value: TimeResponse(dateTime: Date()))
            }
        }

        // swiftlint:disable:next force_try
        let context = try! MullvadApiContext(host: "localhost", address: .ipv4(
            .init(ip: .init("127.0.0.1")!, port: port)
        ), disable_tls: true)
        let proxyFactory = MockProxyFactory.makeProxyFactory(
            transportProvider: transportProvider,
            addressCache: addressCache,
            apiContext: context
        )

        return proxyFactory.createAPIProxy()
    }

    func testSuccessfulResponse() async throws {
        let expectedEndpoints: [AnyIPEndpoint] = [AnyIPEndpoint(string: "12.34.56.78:80")!]

        let bodyData = try encoder.encode(expectedEndpoints)
        let body = String(data: bodyData, encoding: .utf8)!
        let responseCode: UInt = 200
        let mock = mullvad_api_mock_server_response(
            "GET",
            "/app/v1/api-addrs",
            UInt(responseCode),
            body
        )
        defer { mullvad_api_mock_drop(mock) }
        let apiProxy = makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { c in
            _ = apiProxy
                .mullvadApiGetAddressList(
                    retryStrategy: .noRetry
                ) { result in
                    c.resume(returning: result)
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
        let mock = mullvad_api_mock_server_response(
            "GET",
            "/app/v1/api-addrs",
            UInt(expectedResponseCode),
            ""
        )
        defer { mullvad_api_mock_drop(mock) }
        let apiProxy = makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { c in
            _ = apiProxy
                .mullvadApiGetAddressList(
                    retryStrategy: .noRetry
                ) { result in
                    c.resume(returning: result)
                }
        }

        guard let error = result.error as? REST.Error else {
            XCTFail("GetAddressList should have failed with an error.")
            return
        }
        switch error {
        case let .unhandledResponse(responseCode, _):
            XCTAssertEqual(responseCode, expectedResponseCode)
        default:
            XCTFail("GetAddressList failed with the wrong error: \(error)")
        }
    }

    func testInvalidBody() async throws {
        let expectedResponseCode = 200
        let mock = mullvad_api_mock_server_response(
            "GET",
            "/app/v1/api-addrs",
            UInt(expectedResponseCode),
            "This is an invalid JSON"
        )
        defer { mullvad_api_mock_drop(mock) }
        let apiProxy = makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { c in
            _ = apiProxy
                .mullvadApiGetAddressList(
                    retryStrategy: .noRetry
                ) { result in
                    c.resume(returning: result)
                }
        }

        guard let error = result.error as? REST.Error else {
            XCTFail("GetAddressList should have failed with an error.")
            return
        }
        switch error {
        case let .unhandledResponse(responseCode, response):
            XCTAssertNil(response)
            XCTAssertEqual(responseCode, expectedResponseCode)
        default:
            XCTFail("GetAddressList failed with the wrong error: \(error)")
        }
    }

    func testCustomErrorCode() async throws {
        let expectedResponseCode = 400
        let expectedErrorCode = 123
        let mock = mullvad_api_mock_server_response(
            "GET",
            "/app/v1/api-addrs",
            UInt(expectedResponseCode),
            """
            {"code": "\(expectedErrorCode)",
            "error": "A magical error occured"
            }
            """
        )
        defer { mullvad_api_mock_drop(mock) }
        let apiProxy = makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { c in
            _ = apiProxy
                .mullvadApiGetAddressList(
                    retryStrategy: .noRetry
                ) { result in
                    c.resume(returning: result)
                }
        }

        guard let error = result.error as? REST.Error else {
            XCTFail("GetAddressList should have failed with an error.")
            return
        }
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
}
