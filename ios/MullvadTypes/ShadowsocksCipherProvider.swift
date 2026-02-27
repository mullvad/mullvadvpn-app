//
//  ShadowsocksCipherProvider.swift
//  MullvadVPN
//
//  Created by pronebird on 13/11/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging

public struct ShadowsocksCipherProvider {
    public static func getCiphers() -> [String] {
        guard let pointer = get_shadowsocks_chipers() else {
            Logger(label: "ShadowsocksCipher").error("Failed to get shadowsocks ciphers")
            return []
        }

        let cipherString = String(cString: pointer)
        mullvad_api_cstring_drop(pointer)

        return cipherString.components(separatedBy: ",")
    }
}
