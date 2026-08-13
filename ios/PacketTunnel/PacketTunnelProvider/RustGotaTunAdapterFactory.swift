//
//  RustGotaTunAdapterFactory.swift
//  PacketTunnel
//
//  Created by Emīls on 2026-08-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import PacketTunnelCore

/// Factory that creates `RustGotaTunAdapter` instances backed by the Rust FFI.
final class RustGotaTunAdapterFactory: GotaTunAdapterFactory {
    /// Each call to `makeAdapter()` creates a new adapter for one connection attempt.
    func makeAdapter() -> GotaTunAdapterProtocol {
        return RustGotaTunAdapter()
    }
}
