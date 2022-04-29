//
//  SendAppStoreReceiptOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 29/03/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

class SendAppStoreReceiptOperation: ResultOperation<REST.CreateApplePaymentResponse, AppStorePaymentManager.Error> {
    private let restClient: REST.Client
    private let accountToken: String
    private let forceRefresh: Bool
    private let receiptProperties: [String: Any]?
    private var fetchReceiptTask: Cancellable?
    private var submitReceiptTask: Cancellable?

    private let logger = Logger(label: "AppStorePaymentManager.SendAppStoreReceiptOperation")

    init(restClient: REST.Client, accountToken: String, forceRefresh: Bool, receiptProperties: [String: Any]?, completionHandler: @escaping CompletionHandler) {
        self.restClient = restClient
        self.accountToken = accountToken
        self.forceRefresh = forceRefresh
        self.receiptProperties = receiptProperties

        super.init(completionQueue: .main, completionHandler: completionHandler)
    }

    override func cancel() {
        super.cancel()

        DispatchQueue.main.async {
            self.fetchReceiptTask?.cancel()
            self.fetchReceiptTask = nil

            self.submitReceiptTask?.cancel()
            self.submitReceiptTask = nil
        }
    }

    override func main() {
        DispatchQueue.main.async {
            guard !self.isCancelled else {
                self.finish(completion: .cancelled)
                return
            }

            self.fetchReceiptTask = AppStoreReceipt.fetch(forceRefresh: self.forceRefresh, receiptProperties: self.receiptProperties) { completion in
                switch completion {
                case .success(let receiptData):
                    self.sendReceipt(receiptData)

                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to fetch the AppStore receipt.")
                    self.finish(completion: .failure(.readReceipt(error)))

                case .cancelled:
                    self.finish(completion: .cancelled)
                }
            }
        }
    }

    private func sendReceipt(_ receiptData: Data) {
        submitReceiptTask = restClient.createApplePayment(
            token: self.accountToken,
            receiptString: receiptData,
            retryStrategy: .noRetry) { result in
                switch result {
                case .success(let response):
                    self.logger.info("AppStore receipt was processed. Time added: \(response.timeAdded), New expiry: \(response.newExpiry.logFormatDate())")
                    self.finish(completion: .success(response))

                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to send the AppStore receipt.")
                    self.finish(completion: .failure(.sendReceipt(error)))

                case .cancelled:
                    self.logger.debug("Receipt submission cancelled.")
                    self.finish(completion: .cancelled)
                }
            }
    }
}
