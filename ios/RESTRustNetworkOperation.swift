//
//  RESTRustNetworkOperation.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2025-01-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadRustRuntime
import MullvadTypes
import Operations

extension REST {
    typealias RustRequestFactory = ((((MullvadApiResponse) -> Void)?) -> AnyCancellable)

    class RustNetworkOperation<Success: Sendable>: ResultOperation<Success>, @unchecked Sendable {
        private let logger: Logger

        private let requestFactory: RustRequestFactory
        private let responseHandler: ProxyCompletionHandler<[AnyIPEndpoint]>
        private var networkTask: Cancellable?

        private let retryStrategy: RetryStrategy
        private var retryDelayIterator: AnyIterator<Duration>
        private var retryTimer: DispatchSourceTimer?
        private var retryCount = 0

        init(
            name: String,
            dispatchQueue: DispatchQueue,
            retryStrategy: RetryStrategy,
            requestFactory: @escaping RustRequestFactory,
            responseHandler: @escaping ProxyCompletionHandler<[AnyIPEndpoint]>,
            completionHandler: CompletionHandler? = nil
        ) {
            self.retryStrategy = retryStrategy
            retryDelayIterator = retryStrategy.makeDelayIterator()
            self.requestFactory = requestFactory
            self.responseHandler = responseHandler

            var logger = Logger(label: "REST.RustNetworkOperation")
            logger[metadataKey: "name"] = .string(name)
            self.logger = logger

            super.init(
                dispatchQueue: dispatchQueue,
                completionQueue: .main,
                completionHandler: completionHandler
            )
        }

        override public func operationDidCancel() {
            retryTimer?.cancel()
            networkTask?.cancel()

            retryTimer = nil
            networkTask = nil
        }

        override public func main() {
            startRequest()
        }

        func startRequest() {
//            dispatchPrecondition(condition: .onQueue(dispatchQueue))

            guard !isCancelled else {
                finish(result: .failure(OperationError.cancelled))
                return
            }

            networkTask = requestFactory({ [weak self] response in
                guard let self, let body = response.body else { return }

                let decoder = JSONDecoder()
                let decodedResponse = try! decoder.decode([AnyIPEndpoint].self, from: body)

                responseHandler(.success(decodedResponse))
            })
        }
    }
}
