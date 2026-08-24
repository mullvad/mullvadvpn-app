// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadMockData
@preconcurrency import MullvadREST
@preconcurrency import PacketTunnelCore

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
