//
//  PostQuantumKeyReceiving.swift
//  MullvadTypes
//
//  Created by Andrew Bulhak on 2024-03-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import WireGuardKitTypes

protocol PostQuantumKeyReceiving {
    func receivePostQuantumKey(_ key: PrivateKey)
}

enum PostQuantumKeyReceivingError: Error {
    case invalidKey
}

extension PostQuantumKeyReceiving {
    func receivePostQuantumKey(_ keyData: Data) throws {
        guard let key = PrivateKey(rawValue: keyData) else {
            throw PostQuantumKeyReceivingError.invalidKey
        }
        receivePostQuantumKey(key)
    }
}
