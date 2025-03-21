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
            let pointerClass = MullvadApiCompletion { apiResponse in
                try? completion?(apiResponse)
            }

            let rawPointer = Unmanaged.passRetained(pointerClass).toOpaque()

            switch request {
            case let .getAddressList(retryStrategy):
                return MullvadApiCancellable(
                    handle: mullvad_api_get_addresses(
                        apiContext.context,
                        rawPointer,
                        retryStrategy.toRustStrategy()
                    )
                )
            case let .sendProblemReport(retryStrategy, problemReportRequest):
                let rustRequest = RustProblemReportRequest(from: problemReportRequest)

                return MullvadApiCancellable(
                    handle: mullvad_api_send_problem_report(
                        apiContext.context,
                        rawPointer,
                        retryStrategy.toRustStrategy(),
                        rustRequest.getPointer()
                    ),
                    deinitializer: {
                        rustRequest.release()
                    }
                )
            }
        }
    }
}

extension REST {
    public typealias MullvadApiRequestHandler = (((MullvadApiResponse) throws -> Void)?) -> MullvadApiCancellable
}
