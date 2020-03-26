//
//  AppStoreReceipt.swift
//  MullvadVPN
//
//  Created by pronebird on 11/03/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import StoreKit

enum AppStoreReceipt {
    enum Error: Swift.Error {
        /// AppStore receipt file does not exist or file URL is not available
        case doesNotExist

        /// IO error
        case io(Swift.Error)

        /// Failure to refresh the receipt from AppStore
        case refresh(Swift.Error)

        var localizedDescription: String {
            switch self {
            case .doesNotExist:
                return "AppStore receipt file does not exist on disk"
            case .io(let ioError):
                return "Read error: \(ioError.localizedDescription)"
            case .refresh(let refreshError):
                return "Receipt refresh error: \(refreshError.localizedDescription)"
            }
        }
    }

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
    static func fetch(forceRefresh: Bool = false, receiptProperties: [String: Any]? = nil) -> AnyPublisher<Data, Error> {
        let refreshReceiptPublisher = Deferred {
            SKReceiptRefreshRequest(receiptProperties: receiptProperties)
                .publisher
                .mapError { .refresh($0) }
                .flatMap({ _ -> Result<Data, Error>.Publisher in
                    return self.readFromDisk().publisher
                })
        }

        if forceRefresh {
            return refreshReceiptPublisher.eraseToAnyPublisher()
        } else {
            return Deferred { self.readFromDisk().publisher }
                .catch({ (readError) -> AnyPublisher<Data, Error> in
                    // Refresh the receipt from AppStore if it's not on disk
                    if case .doesNotExist = readError {
                        return refreshReceiptPublisher.eraseToAnyPublisher()
                    } else {
                        return Fail(error: readError).eraseToAnyPublisher()
                    }
                })
                .eraseToAnyPublisher()
        }
    }
}
