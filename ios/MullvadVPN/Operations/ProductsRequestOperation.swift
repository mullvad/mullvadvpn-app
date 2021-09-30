//
//  ProductsRequestOperation.swift
//  ProductsRequestOperation
//
//  Created by pronebird on 02/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

class ProductsRequestOperation: AsyncOperation, SKProductsRequestDelegate {
    private let productIdentifiers: Set<String>
    private var completionHandler: ((Result<SKProductsResponse, Error>) -> Void)?

    private let maxRetryCount = 10
    private let retryDelay: DispatchTimeInterval = .seconds(2)

    private var retryCount = 0
    private var retryTimer: DispatchSourceTimer?
    private var request: SKProductsRequest?

    struct OperationCancelledError: LocalizedError {
        var errorDescription: String? {
            return "Operation is cancelled"
        }
    }

    init(productIdentifiers: Set<String>, completionHandler: @escaping (Result<SKProductsResponse, Error>) -> Void) {
        self.productIdentifiers = productIdentifiers
        self.completionHandler = completionHandler

        super.init()
    }

    override func main() {
        DispatchQueue.main.async {
            guard !self.isCancelled else {
                self.finish(with: .failure(OperationCancelledError()))
                return
            }

            self.startRequest()
        }
    }

    override func cancel() {
        super.cancel()

        DispatchQueue.main.async {
            self.request?.cancel()
            self.retryTimer?.cancel()
        }
    }

    // - MARK: SKProductsRequestDelegate

    func requestDidFinish(_ request: SKRequest) {
        // no-op
    }

    func request(_ request: SKRequest, didFailWithError error: Error) {
        DispatchQueue.main.async {
            if self.retryCount < self.maxRetryCount, !self.isCancelled {
                self.retryCount += 1
                self.retry(error: error)
            } else {
                self.finish(with: .failure(error))
            }
        }
    }

    func productsRequest(_ request: SKProductsRequest, didReceive response: SKProductsResponse) {
        DispatchQueue.main.async {
            self.finish(with: .success(response))
        }
    }

    // MARK: - Private

    private func startRequest() {
        request = SKProductsRequest(productIdentifiers: productIdentifiers)
        request?.delegate = self
        request?.start()
    }

    private func retry(error: Error) {
        retryTimer = DispatchSource.makeTimerSource(flags: [], queue: .main)

        retryTimer?.setEventHandler { [weak self] in
            self?.startRequest()
        }

        retryTimer?.setCancelHandler { [weak self] in
            self?.finish(with: .failure(error))
        }

        retryTimer?.schedule(wallDeadline: .now() + self.retryDelay)
        retryTimer?.activate()
    }

    private func finish(with result: Result<SKProductsResponse, Error>) {
        assert(Thread.isMainThread)

        completionHandler?(result)
        completionHandler = nil

        finish()
    }
}
