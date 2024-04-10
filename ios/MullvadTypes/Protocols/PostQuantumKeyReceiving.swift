//
//  PostQuantumKeyReceiving.swift
//  MullvadTypes
//
//  Created by Andrew Bulhak on 2024-03-05.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import WireGuardKitTypes

public protocol PostQuantumKeyReceiving {
    func receivePostQuantumKey(_ key: PreSharedKey?)
}

public enum PostQuantumKeyReceivingError: Error {
    case invalidKey
}

public extension PostQuantumKeyReceiving {
    func receivePostQuantumKey(_ keyData: Data) throws {
        guard let key = PreSharedKey(rawValue: keyData) else {
            throw PostQuantumKeyReceivingError.invalidKey
        }
        receivePostQuantumKey(key)
    }
}
