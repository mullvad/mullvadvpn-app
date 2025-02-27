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

    public init(handle: consuming SwiftCancelHandle) {
        self.handle = handle
    }

    deinit {
        mullvad_api_cancel_task_drop(handle)
    }

    public func cancel() {
        mullvad_api_cancel_task(handle)
    }
}
