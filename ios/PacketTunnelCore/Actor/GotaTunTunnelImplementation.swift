// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST
import MullvadTypes

/// GotaTun tunnel implementation.
/// Unlike WireGuardGo, this implementation does NOT use an external state observer.
/// GotaTunActor may be used directly by the PacketTunnelProvider once we get rid of WireGuardGo.
public final class GotaTunTunnelImplementation: TunnelImplementation, Sendable {
    private let gotaTunActor: GotaTunActor
    public var actor: any PacketTunnelActorProtocol { gotaTunActor }

    public init(
        providerDelegate: sending TunnelProviderDelegate,
        blockedStateErrorMapper: sending BlockedStateErrorMapperProtocol,
        adapterFactory: GotaTunAdapterFactory,
        ipOverrideWrapper: IPOverrideWrapper,
        settingsReader: sending TunnelSettingsManager
    ) {
        gotaTunActor = GotaTunActor(
            providerDelegate: providerDelegate,
            settingsReader: settingsReader,
            relaySelector: RelaySelectorWrapper(relayCache: ipOverrideWrapper),
            blockedStateErrorMapper: blockedStateErrorMapper,
            adapterFactory: adapterFactory
        )
    }

    public func startTunnel(options: StartOptions) async {
        await actor.start(options: options)
    }

    public func stopTunnel() async {
        await actor.stop()
        await actor.waitUntilDisconnected()
    }

    public func sleep() async {
        await actor.onSleep()
    }

    public func wake() {
        actor.onWake()
    }
}
