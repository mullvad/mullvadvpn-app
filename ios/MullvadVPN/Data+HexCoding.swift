//
//  Data+HexCoding.swift
//  MullvadVPN
//
//  Created by pronebird on 20/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Data {
    func hexEncodedString() -> String {
        return map { String(format: "%02hhx", $0) }.joined()
    }
}
