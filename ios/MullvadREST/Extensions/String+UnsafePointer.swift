//
//  String+UnsafePointer.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-19.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation
extension String {
    /// This method safely converts the Swift `String` into a null-terminated C string
    func withCStringPointer() -> UnsafePointer<UInt8>? {
        return withCString { cString in
            // Convert the UnsafePointer<CChar> to UnsafePointer<UInt8>
            let pointer = UnsafePointer<UInt8>(bitPattern: Int(bitPattern: cString))
            return pointer
        }
    }
}
