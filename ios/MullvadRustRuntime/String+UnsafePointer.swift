//
//  String+UnsafePointer.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-04-02.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension String {
    // Ensure the string is converted to a null-terminated C string
    // UnsafeMutablePointer provides no automated memory management or alignment guarantees.
    // The caller is responsible to manage the memory
    func toCStringPointer() -> UnsafePointer<CChar>? {
        // Convert the Swift string to a null-terminated UTF-8 C string
        guard let cString = cString(using: .utf8) else { return nil }

        // Allocate memory for characters + null terminator
        let pointer = UnsafeMutablePointer<CChar>.allocate(capacity: cString.count)

        // Copy the characters (including the null terminator)
        pointer.initialize(from: cString, count: cString.count)

        return UnsafePointer(pointer)
    }
}
