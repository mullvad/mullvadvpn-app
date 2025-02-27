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

            return switch request {
            case .getAddressList:
                MullvadApiCancellable(handle: mullvad_api_get_addresses(apiContext.context, rawPointer))
            }
        }
    }
}

extension REST {
    public typealias MullvadApiRequestHandler = (((MullvadApiResponse) throws -> Void)?) -> MullvadApiCancellable
}
