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
            bootRecoveryPeriodicity: .milliseconds(10),
            wgKeyPropagationDelay: .zero
        )
    }
}

extension PacketTunnelActor {
    static func mock(
        tunnelAdapter: TunnelAdapterProtocol = TunnelAdapterDummy(),
        tunnelMonitor: TunnelMonitorProtocol = TunnelMonitorStub.nonFallible(),
        defaultPathObserver: DefaultPathObserverProtocol = DefaultPathObserverFake(),
        blockedStateErrorMapper: BlockedStateErrorMapperProtocol = BlockedStateErrorMapperStub(),
        relaySelector: RelaySelectorProtocol = RelaySelectorStub.nonFallible(),
        settingsReader: SettingsReaderProtocol = SettingsReaderStub.staticConfiguration()
    ) -> PacketTunnelActor {
        return PacketTunnelActor(
            timings: .timingsForTests,
            tunnelAdapter: tunnelAdapter,
            tunnelMonitor: tunnelMonitor,
            defaultPathObserver: defaultPathObserver,
            blockedStateErrorMapper: blockedStateErrorMapper,
            relaySelector: relaySelector,
            settingsReader: settingsReader,
            protocolObfuscator: ProtocolObfuscationStub()
        )
    }
}
