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

            return switch request {
            case let .getAddressList(retryStrategy):
                MullvadApiCancellable(handle: mullvad_api_get_addresses(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy()
                ))

            case let .getRelayList(retryStrategy, etag: etag):
                MullvadApiCancellable(handle: mullvad_api_get_relays(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    etag
                ))
            case let .sendProblemReport(retryStrategy, problemReportRequest):
                MullvadApiCancellable(handle: mullvad_api_send_problem_report(
                    apiContext.context,
                    rawCompletionPointer,
                    retryStrategy.toRustStrategy(),
                    problemReportRequest.toRust()
                ))
            }
        }
    }
}

extension REST {
    public typealias MullvadApiRequestHandler = (((MullvadApiResponse) throws -> Void)?) -> MullvadApiCancellable
}

private extension REST.ProblemReportRequest {
    func toRust() -> UnsafePointer<SwiftProblemReportRequest> {
        let structPointer = UnsafeMutablePointer<SwiftProblemReportRequest>.allocate(capacity: 1)

        let addressPointer = address.toUnsafePointer()
        let messagePointer = message.toUnsafePointer()
        let logPointer = log.toUnsafePointer()

        structPointer.initialize(to: SwiftProblemReportRequest(
            address: addressPointer,
            address_len: UInt(address.utf8.count),
            message: messagePointer,
            message_len: UInt(message.utf8.count),
            log: logPointer,
            log_len: UInt(log.utf8.count)
        ))

        return UnsafePointer(structPointer)
    }
}
