//
//  MullvadApiCancellable.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-02-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public final class MullvadApiCancellable: Cancellable, Sendable {
    private let handle: SwiftCancelHandle

    public init(handle: consuming SwiftCancelHandle) {
        self.handle = handle
    }

    public func start(_ completion: (@Sendable (SwiftMullvadApiResponse) throws -> Void)?) {
        let completionCookie = MullvadApiCompletion { apiResponse in
            try? completion?(apiResponse)
        }
        mullvadApiStartTask(handle: handle, completionCookie: completionCookie)
    }

    public func cancel() {
        mullvadApiCancelTask(handle: handle)
    }
}
