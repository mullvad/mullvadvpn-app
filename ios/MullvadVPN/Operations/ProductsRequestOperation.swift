//
//  ProductsRequestOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 02/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Operations
import StoreKit

public final class ProductsRequestOperation: ResultOperation<SKProductsResponse, Error>,
    SKProductsRequestDelegate
{
    private let productIdentifiers: Set<String>

    private let maxRetryCount = 10
    private let retryDelay: DispatchTimeInterval = .seconds(2)

    private var retryCount = 0
    private var retryTimer: DispatchSourceTimer?
    private var request: SKProductsRequest?

    init(productIdentifiers: Set<String>, completionHandler: @escaping CompletionHandler) {
        self.productIdentifiers = productIdentifiers

        super.init(
            dispatchQueue: .main,
            completionQueue: .main,
            completionHandler: completionHandler
        )
    }

    override public func main() {
        startRequest()
    }

    override public func operationDidCancel() {
        request?.cancel()
        retryTimer?.cancel()
    }

    // - MARK: SKProductsRequestDelegate

    public func requestDidFinish(_ request: SKRequest) {
        // no-op
    }

    public func request(_ request: SKRequest, didFailWithError error: Error) {
        dispatchQueue.async {
            if self.retryCount < self.maxRetryCount, !self.isCancelled {
                self.retryCount += 1
                self.retry(error: error)
            } else {
                self.finish(completion: .failure(error))
            }
        }
    }

    public func productsRequest(
        _ request: SKProductsRequest,
        didReceive response: SKProductsResponse
    ) {
        finish(completion: .success(response))
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
            self?.finish(completion: .failure(error))
        }

        retryTimer?.schedule(wallDeadline: .now() + retryDelay)
        retryTimer?.activate()
    }
}
