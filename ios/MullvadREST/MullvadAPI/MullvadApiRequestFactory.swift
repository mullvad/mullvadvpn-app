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

    public init(apiContext: MullvadApiContext) {
        self.apiContext = apiContext
    }

    public func makeRequest(_ request: APIRequest) -> REST.MullvadApiRequestHandler {
        { completion in
            let completionPointer = MullvadApiCompletion { apiResponse in
                try? completion?(apiResponse)
            }

            let rawCompletionPointer = Unmanaged.passRetained(completionPointer).toOpaque()

            switch request {
            case let .getAddressList(retryStrategy):
                return MullvadApiCancellable(handle: mullvad_ios_get_addresses(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy()
                ))

            case let .getRelayList(retryStrategy, etag: etag):
                return MullvadApiCancellable(handle: mullvad_ios_get_relays(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    etag
                ))
            case let .sendProblemReport(retryStrategy, problemReportRequest):
                let rustRequest = RustProblemReportRequest(from: problemReportRequest)
                return MullvadApiCancellable(handle: mullvad_ios_send_problem_report(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    rustRequest.toRust()
                ))
            case let .getAccount(retryStrategy, accountNumber: accountNumber):
                return MullvadApiCancellable(handle: mullvad_ios_get_account(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))
            case let .createAccount(retryStrategy):
                return MullvadApiCancellable(handle: mullvad_ios_create_account(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy()
                ))
            case let .deleteAccount(retryStrategy, accountNumber: accountNumber):
                return MullvadApiCancellable(handle: mullvad_ios_delete_account(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))

            // Device Proxy
            case let .getDevice(retryStrategy, accountNumber: accountNumber, identifier):
                return MullvadApiCancellable(handle: mullvad_ios_get_device(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber,
                    identifier
                ))

            case let .getDevices(retryStrategy, accountNumber):
                return MullvadApiCancellable(handle: mullvad_ios_get_devices(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber
                ))

            case let .deleteDevice(retryStrategy, accountNumber, identifier):
                return MullvadApiCancellable(handle: mullvad_ios_delete_device(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber,
                    identifier
                ))
            case let .rotateDeviceKey(retryStrategy, accountNumber, identifier, publicKey):
                return MullvadApiCancellable(handle: mullvad_ios_rotate_device_key(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber,
                    identifier,
                    publicKey.rawValue.map { $0 }
                ))
            case let .createDevice(retryStrategy, accountNumber, request):
                return MullvadApiCancellable(handle: mullvad_ios_create_device(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    accountNumber,
                    request.publicKey.rawValue.map { $0 }
                ))
            }
        }
    }
}

extension REST {
    public typealias MullvadApiRequestHandler = (((MullvadApiResponse) throws -> Void)?) -> MullvadApiCancellable
}
