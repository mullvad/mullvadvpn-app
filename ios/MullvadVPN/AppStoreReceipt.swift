//
//  AppStoreReceipt.swift
//  MullvadVPN
//
//  Created by pronebird on 11/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import StoreKit

enum AppStoreReceipt {
    enum Error: ChainedError {
        /// AppStore receipt file does not exist or file URL is not available
        case doesNotExist

        /// IO error
        case io(Swift.Error)

        /// Failure to refresh the receipt from AppStore
        case refresh(Swift.Error)

        var errorDescription: String? {
            switch self {
            case .doesNotExist:
                return "AppStore receipt file does not exist on disk"
            case .io:
                return "Read error"
            case .refresh:
                return "Receipt refresh error"
            }
        }
    }

    /// An operation queue used to run receipt refresh requests
    private static let operationQueue: OperationQueue = {
        let queue = OperationQueue()
        queue.name = "AppStoreReceiptQueue"
        queue.maxConcurrentOperationCount = 1
        return queue
    }()

    /// Read AppStore receipt from disk
    static func readFromDisk() -> Result<Data, Error> {
        guard let appStoreReceiptURL = Bundle.main.appStoreReceiptURL else {
            return .failure(.doesNotExist)
        }

        return Result { try Data(contentsOf: appStoreReceiptURL) }
            .mapError { (error) -> Error in
                if let ioError = error as? CocoaError, ioError.code == .fileNoSuchFile {
                    return .doesNotExist
                } else {
                    return .io(error)
                }
        }
    }

    /// Read AppStore receipt from disk or refresh it from the AppStore if it's missing
    /// This call may trigger a sign in with AppStore prompt to appear
    static func fetch(forceRefresh: Bool = false, receiptProperties: [String: Any]? = nil,
                      completionHandler: @escaping (Result<Data, Error>) -> Void)
    {
        if forceRefresh {
            refreshReceipt(receiptProperties: receiptProperties,
                           completionHandler: completionHandler)
        } else {
            switch self.readFromDisk() {
            case .success(let data):
                completionHandler(.success(data))

            case .failure(let error):
                // Refresh the receipt from AppStore if it's not on disk
                if case .doesNotExist = error {
                    refreshReceipt(receiptProperties: receiptProperties,
                                   completionHandler: completionHandler)
                } else {
                    completionHandler(.failure(error))
                }
            }
        }
    }

    private static func refreshReceipt(receiptProperties: [String: Any]?, completionHandler: @escaping (Result<Data, Error>) -> Void) {
        let refreshOperation = ReceiptRefreshOperation(receiptProperties: receiptProperties)
        refreshOperation.addDidFinishBlockObserver { (operation, result) in
            let result = result
                .mapError { Error.refresh($0) }
                .flatMap { Self.readFromDisk() }
            completionHandler(result)
        }

        operationQueue.addOperation(refreshOperation)
    }
}


private class ReceiptRefreshOperation: AsyncOperation, OutputOperation, SKRequestDelegate {
    typealias Output = Result<(), Error>

    private let request: SKReceiptRefreshRequest

    init(receiptProperties: [String: Any]?) {
        request = SKReceiptRefreshRequest(receiptProperties: receiptProperties)
    }

    override func main() {
        request.delegate = self
        request.start()
    }

    override func operationDidCancel() {
        request.cancel()
    }

    // - MARK: SKRequestDelegate

    func requestDidFinish(_ request: SKRequest) {
        finish(with: .success(()))
    }

    func request(_ request: SKRequest, didFailWithError error: Error) {
        finish(with: .failure(error))
    }
}
