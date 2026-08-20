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

    public func start(_ completion: ((MullvadApiResponse) throws -> Void)?) {
        let completionPointer = MullvadApiCompletion { apiResponse in
            try? completion?(apiResponse)
        }
        let rawCompletionPointer = Unmanaged.passRetained(completionPointer).toOpaque()
        mullvadApiStartTask(handle: handle, completionCookie: UInt64(Int(bitPattern: rawCompletionPointer)))
    }

    public func cancel() {
        mullvadApiCancelTask(handle: handle)
    }
}
