//
//  RustGotaTunAdapterFactory.swift
//  PacketTunnel
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import PacketTunnelCore

/// Factory that creates `RustGotaTunAdapter` instances backed by the Rust FFI.
/// Each call to `makeAdapter()` creates a new adapter for one connection attempt.
final class RustGotaTunAdapterFactory: GotaTunAdapterFactory {
    private let logger = Logger(label: "RustGotaTunAdapterFactory")

    func makeAdapter() -> GotaTunAdapterProtocol {
        logger.debug("Creating Rust-backed GotaTun adapter")
        return RustGotaTunAdapter()
    }
}
