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
    case checkStorekitPayment(retryStrategy: REST.RetryStrategy, accountNumber: String, transaction: String)
}

struct MullvadApiRequestFactory {
    let apiContext: MullvadApiContext

    func makeRequest(_ request: MullvadApiRequest) -> REST.MullvadApiRequestHandler {
        { completion in
            let pointerClass = MullvadApiCompletion { apiResponse in
                try? completion?(apiResponse)
            }

            let rawPointer = Unmanaged.passRetained(pointerClass).toOpaque()

            return switch request {
            case let .getAddressList(retryStrategy):
                MullvadApiCancellable(handle: mullvad_api_get_addresses(
                    apiContext.context,
                    rawPointer,
                    retryStrategy.toRustStrategy()
                ))
            case let .initStorekitPayment(
                retryStrategy: retryStrategy,
                accountNumber: accountNumber
            ):
                MullvadApiCancellable(handle: mullvad_api_init_storekit_payment(
                    apiContext.context,
                    rawPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))
            case let .checkStorekitPayment(
                retryStrategy: retryStrategy,
                accountNumber: accountNumber,
                transaction: transaction
            ):
                MullvadApiCancellable(handle: mullvad_api_check_storekit_payment(
                    apiContext.context,
                    rawPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber,
                    transaction
                ))
            }
        }
    }
}

extension REST {
    typealias MullvadApiRequestHandler = (((MullvadApiResponse) throws -> Void)?) -> MullvadApiCancellable
}
