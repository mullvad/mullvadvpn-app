//
//  MullvadApiTest.swift
//  MullvadVPN
//
//  Created by Steffen Ernst on 2025-02-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import MullvadRustRuntime
import XCTest

class MullvadApiTests: XCTestCase {
    func testSuccessfulRequestHasNoError() async throws {
        let expectation = expectation(description: "Got a response")

        let expectedBody = "[]"
        let expectedResponseCode: UInt16 = 200
        let mock = mullvad_api_mock_server_response(
            "GET",
            "/app/v1/api-addrs",
            UInt(expectedResponseCode),
            expectedBody
        )

//        _ = await HTTPMockServer(
//            type: "GET",
//            route: "/app/v1/api-addrs",
//            responseCode: expectedResponseCode,
//            responseBody: expectedBody
//        )

        var receivedResponseBody: String?
        var receivedResponseCode: UInt16?
        let pointerClass = MullvadApiCompletion { apiResponse in
            receivedResponseCode = apiResponse.statusCode
            if let data = apiResponse.body {
                receivedResponseBody = String(data: data, encoding: .utf8)
            }
            expectation.fulfill()
        }
        let rawPointer = Unmanaged.passRetained(pointerClass).toOpaque()
        let context = mullvad_api_init_new_tls_disabled(
            "localhost",
            "127.0.0.1:\(mock.port)"
        )

        mullvad_api_get_addresses(context, rawPointer)

        await fulfillment(of: [expectation], timeout: 1)
        XCTAssertEqual(receivedResponseBody, expectedBody)
        XCTAssertEqual(receivedResponseCode, expectedResponseCode)
    }
}
