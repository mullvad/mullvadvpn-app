//
//  MullvadApiRequestFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-02-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import MullvadTypes

public struct MullvadApiRequestFactory: Sendable {
    public let apiContext: MullvadApiContext
    private let encoder: JSONEncoder

    public init(apiContext: MullvadApiContext, encoder: JSONEncoder) {
        self.apiContext = apiContext
        self.encoder = encoder
    }

    // swiftlint:disable:next function_body_length
    public func makeRequest(_ request: APIRequest) -> REST.MullvadApiRequestHandler {
        { completion in
            let completionPointer = MullvadApiCompletion { apiResponse in
                try? completion?(apiResponse)
            }

            let rawCompletionPointer = Unmanaged.passRetained(completionPointer).toOpaque()

            switch request {
            case let .getAddressList(retryStrategy):
                return MullvadApiCancellable(handle: mullvad_api_get_addresses(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy()
                ))

            case let .getRelayList(retryStrategy, etag: etag):
                return MullvadApiCancellable(handle: mullvad_api_get_relays(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    etag
                ))
            case let .getAccount(retryStrategy, accountNumber: accountNumber):
                return MullvadApiCancellable(handle: mullvad_api_get_account(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))
            case let .createAccount(retryStrategy):
                return MullvadApiCancellable(handle: mullvad_api_create_account(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy()
                ))
            case let .deleteAccount(retryStrategy, accountNumber: accountNumber):
                return MullvadApiCancellable(handle: mullvad_api_delete_account(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))
            case let .initStorekitPayment(
                retryStrategy: retryStrategy,
                accountNumber: accountNumber
            ):
                return MullvadApiCancellable(handle: mullvad_ios_init_storekit_payment(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))
            case let .checkStorekitPayment(
                retryStrategy: retryStrategy,
                accountNumber: accountNumber,
                transaction: transaction
            ):
                let body = try encoder.encode(transaction)
                return MullvadApiCancellable(handle: mullvad_ios_check_storekit_payment(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber,
                    body.map { $0 },
                    UInt(body.count)
                ))
            }
        }
    }
}

extension REST {
    public typealias MullvadApiRequestHandler = (((MullvadApiResponse) throws -> Void)?) throws -> MullvadApiCancellable
}
