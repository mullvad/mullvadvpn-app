//
//  SwiftData.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2026-02-12.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

// Code for passing `Data` objects to C FFI code

@_cdecl("swift_data_get_ptr")
func getSwiftDataPtr(_ pointer: UnsafePointer<SwiftData>?) -> UnsafeRawPointer? {
    guard
        let contentPtr = pointer?.pointee.ptr as? UnsafePointer<NSData>
    else { return nil }
    return contentPtr.pointee.bytes
}

@_cdecl("swift_data_get_len")
func getSwiftDataLength(_ pointer: UnsafePointer<SwiftData>?) -> Int {
    guard
        let contentPtr = pointer?.pointee.ptr as? UnsafePointer<NSData>
    else { return 0 }
    return contentPtr.pointee.count
}

@_cdecl("swift_data_drop")
func dropSwiftData(_ pointer: UnsafeMutablePointer<SwiftData>?) {
    // release the NSData
    if let ptr = pointer?.pointee.ptr {
        _ = Unmanaged<NSData>.fromOpaque(ptr).takeRetainedValue()
    }
    pointer?.pointee.ptr = nil
}

extension SwiftData {
    init(data: NSData) {
        self.init(ptr: Unmanaged.passRetained(data).toOpaque())
    }
}
