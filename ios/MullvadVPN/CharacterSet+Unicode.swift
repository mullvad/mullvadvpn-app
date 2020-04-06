//
//  CharacterSet+Unicode.swift
//  MullvadVPN
//
//  Created by pronebird on 08/04/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension CharacterSet {
    func containsUnicodeScalars(of character: Character) -> Bool {
        return character.unicodeScalars.allSatisfy { self.contains($0) }
    }
}
