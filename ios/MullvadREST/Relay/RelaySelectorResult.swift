//
//  RelaySelectorResult.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-05-14.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

public typealias RelaySelectorResult = RelaySelectorMatch

public struct RelaySelectorMatch: Codable, Equatable {
    public var endpoint: MullvadEndpoint
    public var relay: REST.ServerRelay
    public var location: Location
}
