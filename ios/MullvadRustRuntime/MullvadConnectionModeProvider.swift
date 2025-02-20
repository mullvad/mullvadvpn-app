//
//  MullvadConnectionModeProvider.swift
//  MullvadRustRuntime
//
//  Created by Marco Nikic on 2025-02-20.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public func initConnectionModeProvider(provider: SwiftConnectionModeProviderProxy) -> SwiftConnectionModeProvider {
    let rawProvider = Unmanaged.passRetained(provider)
        .toOpaque()
    return init_connection_mode_provider(rawProvider)
}

@_cdecl("connection_mode_provider_initial")
func ConnectionModeProviderInitial(rawPointer: UnsafeMutableRawPointer) {
    let accessMethodIterator = Unmanaged<SwiftConnectionModeProviderProxy>
        .fromOpaque(rawPointer)
        .takeRetainedValue()
    accessMethodIterator.initial()
}
