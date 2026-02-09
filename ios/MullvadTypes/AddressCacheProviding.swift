//
//  AddressCacheProviding.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2025-05-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public protocol AddressCacheProviding: Sendable {
    func getCurrentEndpoint() -> AnyIPEndpoint
}
