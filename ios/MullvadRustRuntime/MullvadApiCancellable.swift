//
//  MullvadApiCancellable.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-02-07.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public final class MullvadApiCancellable: Cancellable, Sendable {
    private let handle: RequestCancelHandle

    public init(handle: consuming RequestCancelHandle) {
        self.handle = handle
    }

    public func start(_ completion: (@Sendable (SwiftMullvadApiResponse) throws -> Void)?) {
        let completionCookie = MullvadApiCompletion { apiResponse in
            try? completion?(apiResponse)
        }
        handle.startTask(completionCookie: completionCookie)
    }

    public func cancel() {
        handle.cancelTask()
    }
}
