//
//  RESTProxy.swift
//  MullvadREST
//
//  Created by pronebird on 20/04/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntime
import MullvadTypes
import Operations

public typealias ProxyCompletionHandler<Success: Sendable> = @Sendable (Result<Success, Swift.Error>) -> Void
