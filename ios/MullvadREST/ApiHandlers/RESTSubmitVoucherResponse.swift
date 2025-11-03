//
//  RESTAPIProxy.swift
//  MullvadREST
//
//  Created by pronebird on 10/07/2020.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntime
import MullvadTypes
import Operations
import WireGuardKitTypes

extension REST {
    public struct SubmitVoucherResponse: Decodable, Sendable {
        public let timeAdded: Int
        public let newExpiry: Date

        public var dateComponents: DateComponents {
            DateComponents(second: timeAdded)
        }
    }
}
