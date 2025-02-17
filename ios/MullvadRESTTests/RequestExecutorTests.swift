//
//  RequestExecutorTests.swift
//  MullvadRESTTests
//
//  Created by pronebird on 25/08/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadMockData
@testable import MullvadREST
@testable import MullvadTypes
import XCTest

@MainActor
final class RequestExecutorTests: XCTestCase {
    let addressCache = REST.AddressCache(canWriteToCache: false, fileCache: MemoryCache())
    nonisolated(unsafe) var timerServerProxy: TimeServerProxy!

    override func setUp() {
        super.setUp()

        let transportProvider = REST.AnyTransportProvider {
            AnyTransport {
                Response(delay: 1, statusCode: 200, value: TimeResponse(dateTime: Date()))
            }
        }

        let apiTransportProvider = REST.AnyAPITransportProvider {
            APITransportStub()
        }

        let proxyFactory = REST.ProxyFactory.makeProxyFactory(
            transportProvider: transportProvider,
            apiTransportProvider: apiTransportProvider,
            addressCache: addressCache
        )
        timerServerProxy = TimeServerProxy(configuration: proxyFactory.configuration)
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

        waitForExpectations(timeout: .UnitTest.timeout)
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

        waitForExpectations(timeout: .UnitTest.timeout)
    }
}

extension RequestExecutorTests {
    final class APITransportStub: APITransportProtocol, Sendable {
        public var name: String {
            "app-transport-dummy"
        }

        public func sendRequest(
            _ request: APIRequest,
            completion: @escaping @Sendable (ProxyAPIResponse) -> Void
        ) -> Cancellable {
            AnyCancellable()
        }
    }
}
