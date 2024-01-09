//
//  RequestExecutorTests.swift
//  MullvadRESTTests
//
//  Created by pronebird on 25/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
@testable import MullvadTypes
import XCTest

final class RequestExecutorTests: XCTestCase {
    let addressCache = REST.AddressCache(canWriteToCache: false, fileCache: MemoryCache())
    var timerServerProxy: TimeServerProxy!

    override func setUp() {
        super.setUp()

        let transportProvider = REST.AnyTransportProvider {
            AnyTransport {
                Response(delay: 1, statusCode: 200, value: TimeResponse(dateTime: Date()))
            }
        }

        let proxyFactory = REST.ProxyFactory.makeProxyFactory(
            transportProvider: transportProvider,
            addressCache: addressCache
        )
        timerServerProxy = proxyFactory.createTimeServerProxy()
    }

    func testExecuteAsync() async throws {
        _ = try await timerServerProxy.getDateTime().execute()
    }

    func testExecuteWithCompletionBlock() throws {
        let expectation = self.expectation(description: "Wait for request to complete.")

        _ = timerServerProxy.getDateTime().execute { result in
            XCTAssertTrue(result.isSuccess)
            expectation.fulfill()
        }

        waitForExpectations(timeout: 2)
    }

    func testCancelAsyncExecution() async throws {
        let task = Task {
            return try await timerServerProxy.getDateTime().execute()
        }
        task.cancel()

        do {
            _ = try await task.value
            XCTFail("Should always throw OperationError.cancelled")
        } catch {
            XCTAssertTrue(error.isOperationCancellationError)
        }
    }

    func testCancelExecutionWithCompletionBlock() {
        let expectation = self.expectation(description: "Wait for request to complete.")

        let cancellationToken = timerServerProxy.getDateTime().execute { result in
            let isCancellationError = result.error?.isOperationCancellationError ?? false
            XCTAssertTrue(isCancellationError)
            expectation.fulfill()
        }

        cancellationToken.cancel()

        waitForExpectations(timeout: 2)
    }
}
