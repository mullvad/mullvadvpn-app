//
//  URLSessionTransport.swift
//  MullvadTransport
//
//  Created by Mojgan on 2023-12-08.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

struct URLSessionTaskWrapper: Cancellable {
    let task: URLSessionTask
    func cancel() {
        task.cancel()
    }
}
