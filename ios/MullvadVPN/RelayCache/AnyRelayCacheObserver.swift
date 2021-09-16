//
//  AnyRelayCacheObserver.swift
//  AnyRelayCacheObserver
//
//  Created by pronebird on 09/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension RelayCache {

    final class AnyRelayCacheObserver: WeakObserverBox, RelayCacheObserver {
        typealias Wrapped = RelayCacheObserver

        private(set) weak var inner: RelayCacheObserver?

        init<T: RelayCacheObserver>(_ inner: T) {
            self.inner = inner
        }

        func relayCache(_ relayCache: RelayCache.Tracker, didUpdateCachedRelays cachedRelays: CachedRelays) {
            inner?.relayCache(relayCache, didUpdateCachedRelays: cachedRelays)
        }

        static func == (lhs: AnyRelayCacheObserver, rhs: AnyRelayCacheObserver) -> Bool {
            return lhs.inner === rhs.inner
        }
    }

}
