//
//  CharacterSet+IPAddress.swift
//  MullvadVPN
//
//  Created by pronebird on 07/10/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension CharacterSet {
    static var ipv4AddressCharset: CharacterSet {
        return CharacterSet(charactersIn: "0123456789.")
    }

    static var ipv6AddressCharset: CharacterSet {
        return CharacterSet(charactersIn: "0123456789abcdef:.")
    }
}
