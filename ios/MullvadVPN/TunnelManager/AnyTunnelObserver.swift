//
//  AnyTunnelObserver.swift
//  AnyTunnelObserver
//
//  Created by pronebird on 19/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

class AnyTunnelObserver: WeakObserverBox, TunnelObserver {

    typealias Wrapped = TunnelObserver

    private(set) weak var inner: TunnelObserver?

    init<T: TunnelObserver>(_ observer: T) {
        inner = observer
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelState tunnelState: TunnelState) {
        inner?.tunnelManager(manager, didUpdateTunnelState: tunnelState)
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelInfo: TunnelInfo?) {
        inner?.tunnelManager(manager, didUpdateTunnelSettings: tunnelInfo)
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: TunnelManager.Error) {
        inner?.tunnelManager(manager, didFailWithError: error)
    }

    static func == (lhs: AnyTunnelObserver, rhs: AnyTunnelObserver) -> Bool {
        return lhs.inner === rhs.inner
    }
}
