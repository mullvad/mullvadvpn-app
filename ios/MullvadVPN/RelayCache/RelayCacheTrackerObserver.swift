//
//  RelayCacheTrackerObserver.swift
//  RelayCacheTrackerObserver
//
//  Created by pronebird on 09/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import RelayCache

protocol RelayCacheTrackerObserver: AnyObject {
    func relayCacheTracker(
        _ tracker: RelayCacheTracker,
        didUpdateCachedRelays cachedRelays: CachedRelays
    )
}
