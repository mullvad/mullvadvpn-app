//
//  Array+.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-26.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension Array where Element: NSObjectProtocol {
    mutating func removeFirst(where predicate: (Element) -> Bool) {
        if let index = firstIndex(where: predicate) {
            remove(at: index)
        }
    }
}
