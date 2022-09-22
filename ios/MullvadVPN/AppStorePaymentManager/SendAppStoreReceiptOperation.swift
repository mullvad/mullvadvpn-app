//
//  SendAppStoreReceiptOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 29/03/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

class SendAppStoreReceiptOperation: ResultOperation<
    REST.CreateApplePaymentResponse,
    AppStorePaymentManager.Error
> {
    private let apiProxy: REST.APIProxy
    private let accountToken: String
    private let forceRefresh: Bool
    private let receiptProperties: [String: Any]?
    private var fetchReceiptTask: Cancellable?
    private var submitReceiptTask: Cancellable?

    private let logger = Logger(label: "SendAppStoreReceiptOperation")

    init(
        apiProxy: REST.APIProxy,
        accountToken: String,
        forceRefresh: Bool,
        receiptProperties: [String: Any]?,
        completionHandler: @escaping CompletionHandler
    ) {
        self.apiProxy = apiProxy
        self.accountToken = accountToken
        self.forceRefresh = forceRefresh
        self.receiptProperties = receiptProperties

        super.init(
            dispatchQueue: .main,
            completionQueue: .main,
            completionHandler: completionHandler
        )
    }

    override func operationDidCancel() {
        fetchReceiptTask?.cancel()
        fetchReceiptTask = nil

        submitReceiptTask?.cancel()
        submitReceiptTask = nil
    }

    override func main() {
        fetchReceiptTask = AppStoreReceipt.fetch(
            forceRefresh: forceRefresh,
            receiptProperties: receiptProperties
        ) { completion in
            switch completion {
            case let .success(receiptData):
                self.sendReceipt(receiptData)

            case let .failure(error):
                self.logger.error(
                    error: error,
                    message: "Failed to fetch the AppStore receipt."
                )
                self.finish(completion: .failure(.readReceipt(error)))

            case .cancelled:
                self.finish(completion: .cancelled)
            }
        }
    }

    private func sendReceipt(_ receiptData: Data) {
        submitReceiptTask = apiProxy.createApplePayment(
            accountNumber: accountToken,
            receiptString: receiptData,
            retryStrategy: .noRetry
        ) { result in
            switch result {
            case let .success(response):
                self.logger
                    .info(
                        "AppStore receipt was processed. Time added: \(response.timeAdded), New expiry: \(response.newExpiry.logFormatDate())"
                    )
                self.finish(completion: .success(response))

            case let .failure(error):
                self.logger.error(
                    error: error,
                    message: "Failed to send the AppStore receipt."
                )
                self.finish(completion: .failure(.sendReceipt(error)))

            case .cancelled:
                self.logger.debug("Receipt submission cancelled.")
                self.finish(completion: .cancelled)
            }
        }
    }
}
