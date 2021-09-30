//
//  ReceiptRefreshOperation.swift
//  ReceiptRefreshOperation
//
//  Created by pronebird on 02/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

class ReceiptRefreshOperation: AsyncOperation, SKRequestDelegate {
    private let request: SKReceiptRefreshRequest
    private var completionHandler: ((Result<(), Error>) -> Void)?

    struct OperationCancelledError: LocalizedError {
        var errorDescription: String? {
            return "Operation is cancelled"
        }
    }

    init(receiptProperties: [String: Any]?, completionHandler completion: @escaping (Result<(), Error>) -> Void) {
        request = SKReceiptRefreshRequest(receiptProperties: receiptProperties)
        completionHandler = completion
    }

    override func main() {
        DispatchQueue.main.async {
            guard !self.isCancelled else {
                self.finish(with: .failure(OperationCancelledError()))
                return
            }

            self.request.delegate = self
            self.request.start()
        }
    }

    override func cancel() {
        DispatchQueue.main.async {
            super.cancel()

            self.request.cancel()
        }
    }

    // - MARK: SKRequestDelegate

    func requestDidFinish(_ request: SKRequest) {
        DispatchQueue.main.async {
            self.finish(with: .success(()))
        }
    }

    func request(_ request: SKRequest, didFailWithError error: Error) {
        DispatchQueue.main.async {
            self.finish(with: .failure(error))
        }
    }

    private func finish(with result: Result<(), Error>) {
        assert(Thread.isMainThread)

        completionHandler?(result)
        completionHandler = nil

        finish()
    }
}
