//
//  Array+.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-26.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Array where Element: NSObjectProtocol {
    mutating func removeFirstInstance(of type: AnyClass) {
        if let index = firstIndex(where: { $0.isKind(of: type.self) }) {
            remove(at: index)
        }
    }
}
