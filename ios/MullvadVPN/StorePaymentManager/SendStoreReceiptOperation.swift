//
//  SendStoreReceiptOperation.swift
//  MullvadVPN
//
//  Created by pronebird on 29/03/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import StoreKit

class SendStoreReceiptOperation: ResultOperation<
    REST.CreateApplePaymentResponse,
    StorePaymentManagerError
>, SKRequestDelegate {
    private let apiProxy: REST.APIProxy
    private let accountNumber: String

    private let forceRefresh: Bool
    private let receiptProperties: [String: Any]?
    private var refreshRequest: SKReceiptRefreshRequest?

    private var submitReceiptTask: Cancellable?

    private let logger = Logger(label: "SendStoreReceiptOperation")

    init(
        apiProxy: REST.APIProxy,
        accountNumber: String,
        forceRefresh: Bool,
        receiptProperties: [String: Any]?,
        completionHandler: @escaping CompletionHandler
    ) {
        self.apiProxy = apiProxy
        self.accountNumber = accountNumber
        self.forceRefresh = forceRefresh
        self.receiptProperties = receiptProperties

        super.init(
            dispatchQueue: .main,
            completionQueue: .main,
            completionHandler: completionHandler
        )
    }

    override func operationDidCancel() {
        refreshRequest?.cancel()
        refreshRequest = nil

        submitReceiptTask?.cancel()
        submitReceiptTask = nil
    }

    override func main() {
        // Pull receipt from AppStore if requested.
        guard !forceRefresh else {
            startRefreshRequest()
            return
        }

        // Read AppStore receipt from disk.
        do {
            let data = try readReceiptFromDisk()

            sendReceipt(data)
        } catch is StoreReceiptNotFound {
            // Pull receipt from AppStore if it's not cached locally.
            startRefreshRequest()
        } catch {
            logger.error(
                error: error,
                message: "Failed to read the AppStore receipt."
            )
            finish(completion: .failure(.readReceipt(error)))
        }
    }

    // - MARK: SKRequestDelegate

    func requestDidFinish(_ request: SKRequest) {
        dispatchQueue.async {
            do {
                let data = try self.readReceiptFromDisk()

                self.sendReceipt(data)
            } catch {
                self.logger.error(
                    error: error,
                    message: "Failed to read the AppStore receipt after refresh."
                )
                self.finish(completion: .failure(.readReceipt(error)))
            }
        }
    }

    func request(_ request: SKRequest, didFailWithError error: Error) {
        dispatchQueue.async {
            self.logger.error(
                error: error,
                message: "Failed to refresh the AppStore receipt."
            )
            self.finish(completion: .failure(.readReceipt(error)))
        }
    }

    // MARK: - Private

    private func startRefreshRequest() {
        let refreshRequest = SKReceiptRefreshRequest(receiptProperties: receiptProperties)
        refreshRequest.delegate = self
        refreshRequest.start()

        self.refreshRequest = refreshRequest
    }

    private func readReceiptFromDisk() throws -> Data {
        guard let appStoreReceiptURL = Bundle.main.appStoreReceiptURL else {
            throw StoreReceiptNotFound()
        }

        do {
            return try Data(contentsOf: appStoreReceiptURL)
        } catch let error as CocoaError
            where error.code == .fileReadNoSuchFile || error.code == .fileNoSuchFile
        {
            throw StoreReceiptNotFound()
        } catch {
            throw error
        }
    }

    private func sendReceipt(_ receiptData: Data) {
        submitReceiptTask = apiProxy.createApplePayment(
            accountNumber: accountNumber,
            receiptString: receiptData,
            retryStrategy: .noRetry
        ) { completion in
            switch completion {
            case let .success(response):
                self.logger.info(
                    """
                    AppStore receipt was processed. \
                    Time added: \(response.timeAdded), \
                    New expiry: \(response.newExpiry.logFormatDate())
                    """
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

struct StoreReceiptNotFound: LocalizedError {
    var errorDescription: String? {
        return "AppStore receipt file does not exist on disk."
    }
}
