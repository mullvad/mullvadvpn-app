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
            case let .sendProblemReport(retryStrategy, problemReportRequest):
                let rustRequest = RustProblemReportRequest(from: problemReportRequest)
                return MullvadApiCancellable(
                    handle: mullvad_api_send_problem_report(
                        apiContext.context,
                        rawCompletionPointer,
                        retryStrategy.toRustStrategy(),
                        rustRequest.toRust()
                    )
                )
            }
        }
    }
}

extension REST {
    public typealias MullvadApiRequestHandler = (((MullvadApiResponse) throws -> Void)?) -> MullvadApiCancellable
}
