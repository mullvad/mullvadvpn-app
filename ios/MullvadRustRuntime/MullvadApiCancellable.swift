//
//  MullvadApiCancellable.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-02-07.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public class MullvadApiCancellable: Cancellable {
    private let handle: SwiftCancelHandle
    private let deinitializer: (() -> Void)?

    public init(handle: consuming SwiftCancelHandle, deinitializer: (() -> Void)? = nil) {
        self.handle = handle
        self.deinitializer = deinitializer
    }

    deinit {
        deinitializer?()
        mullvad_api_cancel_task_drop(handle)
    }

    public func cancel() {
        mullvad_api_cancel_task(handle)
    }
}
