//
//  RelayCacheObserver.swift
//  RelayCacheObserver
//
//  Created by pronebird on 09/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol RelayCacheObserver: AnyObject {
    func relayCache(
        _ relayCache: RelayCache.Tracker,
        didUpdateCachedRelays cachedRelays: RelayCache.CachedRelays
    )
}
