//
//  NetworkExtension+HiddenSymbols.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2024-12-04.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension

#if swift(>=6)
#if compiler(>=6)
public typealias NWTCPConnection = __NWTCPConnection
public typealias NWHostEndpoint = __NWHostEndpoint
#endif
#endif
