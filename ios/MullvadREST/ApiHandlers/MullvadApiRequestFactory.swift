//
//  MullvadApiRequestFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-02-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import MullvadTypes

enum MullvadApiRequest {
    case getAddressList(retryStrategy: REST.RetryStrategy)
    case initStorekitPayment(retryStrategy: REST.RetryStrategy, accountNumber: String)
    case checkStorekitPayment(retryStrategy: REST.RetryStrategy, accountNumber: String, transaction: StorekitTransaction)
}

struct MullvadApiRequestFactory {
    let apiContext: MullvadApiContext
    let encoder: JSONEncoder

    func makeRequest(_ request: MullvadApiRequest) -> REST.MullvadApiRequestHandler {
        { completion in
            switch request {
            case let .getAddressList(retryStrategy):
                return MullvadApiCancellable(handle: mullvad_api_get_addresses(
                    apiContext.context,
                    makeRawPointer(completion),
                    retryStrategy.toRustStrategy()
                ))
            case let .initStorekitPayment(
                retryStrategy: retryStrategy,
                accountNumber: accountNumber
            ):
                return MullvadApiCancellable(handle: mullvad_api_init_storekit_payment(
                    apiContext.context,
                    makeRawPointer(completion),
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))
            case let .checkStorekitPayment(
                retryStrategy: retryStrategy,
                accountNumber: accountNumber,
                transaction: transaction
            ):
                let body = try encoder.encode(transaction)
                return MullvadApiCancellable(handle: mullvad_api_check_storekit_payment(
                    apiContext.context,
                    makeRawPointer(completion),
                    retryStrategy.toRustStrategy(),
                    accountNumber,
                    body.map { $0 },
                    UInt(body.count)
                ))
            }
        }
    }

    /// This pointer must be consumed
    func makeRawPointer(_ completion: ((MullvadApiResponse) throws -> Void)?) -> UnsafeMutableRawPointer {
        let pointerClass = MullvadApiCompletion { apiResponse in
            try? completion?(apiResponse)
        }

        return Unmanaged.passRetained(pointerClass).toOpaque()
    }
}

extension REST {
    typealias MullvadApiRequestHandler = (((MullvadApiResponse) throws -> Void)?) throws -> MullvadApiCancellable
}
