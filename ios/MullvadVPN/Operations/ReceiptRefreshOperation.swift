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
    private let completionHandler: (Result<(), Error>) -> Void

    init(receiptProperties: [String: Any]?, completionHandler: @escaping (Result<(), Error>) -> Void) {
        request = SKReceiptRefreshRequest(receiptProperties: receiptProperties)
        self.completionHandler = completionHandler

        super.init()

        request.delegate = self
    }

    override func main() {
        request.start()
    }

    override func cancel() {
        super.cancel()

        request.cancel()
    }

    // - MARK: SKRequestDelegate

    func requestDidFinish(_ request: SKRequest) {
        completionHandler(.success(()))
        finish()
    }

    func request(_ request: SKRequest, didFailWithError error: Error) {
        completionHandler(.failure(error))
        finish()
    }
}
