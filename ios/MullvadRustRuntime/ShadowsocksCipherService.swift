//
//  ShadowsocksCipherService.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 10/3/2026.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadTypes

public struct ShadowsocksCipherService {
    public init() {}

    public func getCiphers() -> [String] {
        guard let pointer = get_shadowsocks_chipers() else {
            Logger(label: "ShadowsocksCipherService").error("Failed to get Shadowsocks ciphers")
            return []
        }

        let cipherString = String(cString: pointer)
        mullvad_api_cstring_drop(pointer)

        return cipherString.components(separatedBy: ",").sorted()
    }
}
