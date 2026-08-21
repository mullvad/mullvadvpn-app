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
import MullvadTypes
import PacketTunnelCore

/// Dummy tunnel adapter that does nothing and reports no errors.
class TunnelAdapterDummy: TunnelAdapterProtocol, @unchecked Sendable {
    func apply(settings: PacketTunnelCore.TunnelInterfaceSettings) async throws {}

    func startMultihop(
        entryConfiguration: TunnelAdapterConfiguration?,
        exitConfiguration: TunnelAdapterConfiguration,
        daita: DaitaConfiguration?
    ) async throws {}

    func start(configuration: TunnelAdapterConfiguration, daita: DaitaConfiguration?) async throws {}

    func stop() async throws {}

    func update(configuration: TunnelAdapterConfiguration) async throws {}
}
