//
//  Actor+ConnectionMonitoring.swift
//  PacketTunnelCore
//
//  Created by pronebird on 26/09/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension PacketTunnelActor {
    func listenForTunnelMonitorEvents() async {
        tunnelMonitorTask?.cancel()

        tunnelMonitorTask = Task { [weak self] in
            guard let self else { return }
            for await event in self.tunnelMonitor.eventStream {
                self.eventChannel.send(.monitorEvent(event))
            }
        }
    }
}
