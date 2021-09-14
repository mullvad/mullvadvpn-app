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

    /// Read AppStore receipt from disk or refresh it from AppStore if it's missing.
    /// This call may trigger a sign in with AppStore prompt to appear.
    static func fetch(forceRefresh: Bool = false, receiptProperties: [String: Any]? = nil) -> Result<Data, Error>.Promise {
        if forceRefresh {
            return refreshReceipt(receiptProperties: receiptProperties)
        } else {
            return self.readFromDisk()
                .asPromise()
                .flatMapErrorThen { error in
                    if case .doesNotExist = error {
                        return refreshReceipt(receiptProperties: receiptProperties)
                    } else {
                        return .failure(error)
                    }
                }
        }
    }

    /// Read AppStore receipt from disk
    private static func readFromDisk() -> Result<Data, Error> {
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

    /// Refresh receipt from AppStore
    private static func refreshReceipt(receiptProperties: [String: Any]?) -> Result<Data, Error>.Promise {
        return Result<(), Swift.Error>.Promise { resolver in
            let operation = ReceiptRefreshOperation(receiptProperties: receiptProperties) { result in
                resolver.resolve(value: result)
            }
            self.operationQueue.addOperation(operation)
        }
        .mapError { error in
            return .refresh(error)
        }
        .flatMap {
            return Self.readFromDisk()
        }
    }
}

