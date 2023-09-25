//
//  PacketTunnelActor+Mocks.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 25/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore

extension PacketTunnelActorTimings {
    static var timingsForTests: PacketTunnelActorTimings {
        return PacketTunnelActorTimings(
            bootRecoveryPeriodicity: .milliseconds(100),
            wgKeyPropagationDelay: .milliseconds(100),
            reconnectDebounce: .milliseconds(100)
        )
    }
}

extension PacketTunnelActor {
    static func mock(
        tunnelAdapter: TunnelAdapterProtocol = MockTunnelAdapter(),
        tunnelMonitor: TunnelMonitorProtocol = MockTunnelMonitor.nonFallible(),
        defaultPathObserver: DefaultPathObserverProtocol = MockDefaultPathObserver(),
        blockedStateErrorMapper: BlockedStateErrorMapperProtocol = MockBlockedStateErrorMapper.mock(),
        relaySelector: RelaySelectorProtocol = MockRelaySelector.nonFallible(),
        settingsReader: SettingsReaderProtocol = MockSettingsReader.staticConfiguration()
    ) -> PacketTunnelActor {
        return PacketTunnelActor(
            timings: .timingsForTests,
            tunnelAdapter: tunnelAdapter,
            tunnelMonitor: tunnelMonitor,
            defaultPathObserver: defaultPathObserver,
            blockedStateErrorMapper: blockedStateErrorMapper,
            relaySelector: relaySelector,
            settingsReader: settingsReader
        )
    }
}
