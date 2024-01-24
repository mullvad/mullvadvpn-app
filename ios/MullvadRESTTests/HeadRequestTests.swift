//
//  HeadRequestTests.swift
//  MullvadRESTTests
//
//  Created by Marco Nikic on 2024-01-22.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
import XCTest

class HeadRequestTests: XCTestCase {
    func testSuccessfulRequestHasNoError() throws {
        let transport = RESTTransportStub(data: Data(), response: HTTPURLResponse())
        let request = REST.APIAvailabilityTestRequest(transport: transport)

        let successfulRequestExpectation = expectation(description: "HEAD request completed")
        _ = request.makeRequest { error in
            if error == nil {
                successfulRequestExpectation.fulfill()
            }
        }

        wait(for: [successfulRequestExpectation], timeout: 1)
    }

    func testRequestWithErrors() throws {
        let transport = RESTTransportStub(error: URLError(.timedOut))
        let request = REST.APIAvailabilityTestRequest(transport: transport)

        let failedRequestExpectation = expectation(description: "HEAD request failed")
        _ = request.makeRequest { error in
            if error != nil {
                failedRequestExpectation.fulfill()
            }
        }

        wait(for: [failedRequestExpectation], timeout: 1)
    }
}
