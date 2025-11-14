//
//  MullvadApiTests.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-02-27.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadMockData
import MullvadRustRuntime
import MullvadTypes
import Network
import XCTest

@testable import MullvadREST

/// This tests main purpose is to test the functionallity of the FFI rather than every function of the proxy itself.
/// It makes sure the response and errors are parsed correctly.

class MullvadApiTests: XCTestCase {
    let encoder = JSONEncoder()
    let addressCache = REST.AddressCache(canWriteToCache: false, fileCache: MemoryCache())

    func makeApiProxy(port: UInt16) throws -> APIQuerying {
        let shadowsocksLoader = ShadowsocksLoaderStub(
            configuration: ShadowsocksConfiguration(
                address: .ipv4(.loopback),
                port: 1080,
                password: "123",
                cipher: CipherIdentifiers.CHACHA20.description
            ))

        let accessMethodsRepository = AccessMethodRepositoryStub.stub

        let context = try MullvadApiContext(
            host: "localhost",
            address: "\(IPv4Address.loopback.debugDescription):\(port)",
            domain: REST.encryptedDNSHostname,
            disableTls: true,
            shadowsocksProvider: shadowsocksLoader,
            accessMethodWrapper: initAccessMethodSettingsWrapper(
                methods:
                    accessMethodsRepository
                    .fetchAll()
            ),
            addressCacheProvider: addressCache,
            accessMethodChangeListeners: []
        )

        let proxy = REST.MullvadAPIProxy(
            transportProvider: APITransportProvider(
                requestFactory: .init(
                    apiContext: context,
                    encoder: JSONEncoder()
                )
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

        let mock = MullvadApiMock.get(path: "/app/v1/api-addrs", responseCode: 200, responseData: body)
        let apiProxy = try makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { continuation in
            _ =
                apiProxy
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
        let mock = MullvadApiMock.get(
            path: "/app/v1/api-addrs", responseCode: UInt(expectedResponseCode), responseData: "")
        let apiProxy = try makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { continuation in
            _ =
                apiProxy
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
        let mock = MullvadApiMock.get(
            path: "/app/v1/api-addrs", responseCode: 200, responseData: "This is an invalid JSON"
        )
        let apiProxy = try makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { continuation in
            _ =
                apiProxy
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
        let mock = MullvadApiMock.get(
            path: "/app/v1/api-addrs", responseCode: UInt(expectedResponseCode),
            responseData:
                """
                {"code": "\(expectedErrorCode)",
                "error": "A magical error occurred"
                }
                """
        )
        let apiProxy = try makeApiProxy(port: mock.port)

        let result: Result<[AnyIPEndpoint], Error> = await withCheckedContinuation { continuation in
            _ =
                apiProxy
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

    // This test makes sure the body gets encoded correct.
    func testSuccessfulPostRequest() async throws {
        let problemReportRequest = ProblemReportRequest(
            address: "test@email.com",
            message: "This test should succeed",
            log: "A long log string",
            metadata: [:]
        )

        // The mock server will only responde to requests with `matchBodyString` as body.
        let matchBodyString = String(data: try encoder.encode(problemReportRequest), encoding: .utf8)!
        let expectedResponseCode: UInt = 204
        let mock = MullvadApiMock.post(
            path: "/app/v1/problem-report", responseCode: expectedResponseCode, responseData: matchBodyString)
        let apiProxy = try makeApiProxy(port: mock.port)

        let result: Result<Void, Error> = await withCheckedContinuation { continuation in
            _ =
                apiProxy
                .sendProblemReport(
                    problemReportRequest,
                    retryStrategy:
                        .noRetry
                ) { result in
                    continuation.resume(returning: result)
                }
        }

        XCTAssertNil(result.error)
    }
}
