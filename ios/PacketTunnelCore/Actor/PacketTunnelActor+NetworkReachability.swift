//
//  Actor+NetworkReachability.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network

extension PacketTunnelActor {
    /**
     Start observing changes to default path.

     - Parameter notifyObserverWithCurrentPath: immediately notifies path observer with the current path when set to `true`.
     */
    func startDefaultPathObserver() {
        logger.trace("Start default path observer.")

        defaultPathObserver.start { [weak self] networkPath in
            self?.eventChannel.send(.networkReachability(networkPath))
        }
    }

    /// Stop observing changes to default path.
    func stopDefaultPathObserver() {
        logger.trace("Stop default path observer.")

        defaultPathObserver.stop()
    }

    /**
     Event handler that receives new network path from tunnel monitor and updates internal state with new network reachability status.

     - Parameter networkPath: new default path
     */
    func handleDefaultPathChange(_ networkPath: Network.NWPath.Status) {
        tunnelMonitor.handleNetworkPathUpdate(networkPath)

        let newReachability = networkPath.networkReachability

        state.mutateAssociatedData { $0.networkReachability = newReachability }
    }
}
