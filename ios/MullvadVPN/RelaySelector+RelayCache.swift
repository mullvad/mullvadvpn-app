//
//  RelaySelector+RelayCache.swift
//  MullvadVPN
//
//  Created by pronebird on 07/11/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation

extension RelaySelector {

    static func loadedFromRelayCache() -> AnyPublisher<RelaySelector, RelayCacheError> {
        return RelayCache.withDefaultLocationAndEphemeralSession().publisher
            .flatMap { $0.read() }
            .map { RelaySelector(relayList: $0.relayList) }
            .eraseToAnyPublisher()
    }

}
