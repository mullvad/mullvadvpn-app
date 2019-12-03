//
//  TunnelConfigurationCoder.swift
//  MullvadVPN
//
//  Created by pronebird on 02/10/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation

/// Tunnel configuration encoding and decoding helper
enum TunnelConfigurationCoder {}

extension TunnelConfigurationCoder {

    enum Error: Swift.Error {
        case encode(Swift.Error)
        case decode(Swift.Error)
    }

    static func decode(data: Data) -> Result<TunnelConfiguration, Error> {
        return Result { try JSONDecoder().decode(TunnelConfiguration.self, from: data) }
            .mapError { Error.decode($0) }
    }

    static func encode(tunnelConfig: TunnelConfiguration) -> Result<Data, Error> {
        return Result { try JSONEncoder().encode(tunnelConfig) }
            .mapError { Error.encode($0) }
    }
}
