//
//  AbstractTunData.swift
//  PacketTunnel
//
//  Created by Emils on 02/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation


class DataArray {
    public var arr: [Data]

    init(_ arr: [Data]) {
        self.arr = arr
    }

    func append(_ data: Data) {
        arr.append(data)
    }
    
    func len() -> UInt64 {
        UInt64(arr.count)
    }
    
    static func fromRawPtrUnretained(_ ptr: UnsafeMutableRawPointer) -> DataArray {
        let arr = Unmanaged<DataArray>.fromOpaque(ptr).takeUnretainedValue()
        return arr
    }
    
    static func fromRawPtr(_ ptr: UnsafeMutableRawPointer) -> DataArray {
        let arr = Unmanaged<DataArray>.fromOpaque(ptr).takeRetainedValue()
        return arr
    }
    
    func toRaw() -> UnsafeMutableRawPointer {
        let unmanaged = Unmanaged<DataArray>.passRetained(self)
        let ptr = unmanaged.toOpaque()
        return  ptr
    }
}

@_cdecl("swift_data_array_create")
func dataArrayCreate() -> UnsafeMutableRawPointer {
    let data = DataArray([])
    return data.toRaw()
}

@_cdecl("swift_data_array_append")
func dataArrayAppend(ptr: UnsafeMutableRawPointer, dataPtr: UnsafeRawPointer, dataLen: UInt ) {
    let arr = DataArray.fromRawPtrUnretained(ptr)
    let data = Data(bytes: dataPtr, count: Int(dataLen))
    arr.append(data)
}

@_cdecl("swift_data_array_drop")
func dataArrayDrop(ptr: UnsafeRawPointer) {
    let data = Unmanaged<DataArray>.fromOpaque(ptr)
    data.release()
}

@_cdecl("swift_data_array_len")
func dataArrayLen(ptr: UnsafeMutableRawPointer?) -> UInt64 {
    guard let ptr else {
        return 0
    }
    let arr = DataArray.fromRawPtrUnretained(ptr)
    return arr.len()
}

@_cdecl("swift_data_array_get")
func dataArrayGet(ptr: UnsafeMutableRawPointer, idx: UInt64) -> SwiftData {
    let dataArray = DataArray.fromRawPtrUnretained(ptr)
    let data = dataArray.arr[Int(idx)]
    let dataPtr = (data as NSData).bytes.assumingMemoryBound(to: UInt8.self)
    let mutatingDataPtr = UnsafeMutablePointer(mutating: dataPtr)
    return SwiftData(ptr: mutatingDataPtr, len: UInt(data.count))
}

extension IOOutput {
    mutating func udpV4Traffic() -> [Data] {
        guard self.udp_v4_output != nil else { return [] }
        let data = self.extractData(ptr: self.udp_v4_output)
        self.udp_v4_output = nil
        return data
    }
    
    mutating func udpV6Traffic() -> [Data] {
        guard self.udp_v6_output != nil else { return [] }
        let data = self.extractData(ptr: self.udp_v6_output)
        self.udp_v6_output = nil
        return data
    }   
    
    mutating func hostV4Traffic() -> [Data] {
        guard self.tun_v4_output != nil else { return [] }
        let data = self.extractData(ptr: self.tun_v4_output)
        self.tun_v4_output = nil
        return data
    }
    
    mutating func hostV6Traffic() -> [Data] {
        guard self.tun_v6_output != nil else { return [] }
        let data = self.extractData(ptr: self.tun_v6_output)
        self.tun_v6_output = nil
        return data
    }
    
    mutating func discard() {
        let _ = self.hostV4Traffic()
        let _ = self.hostV6Traffic()
        let _ = self.udpV4Traffic()
        let _ = self.udpV6Traffic()
    }
    
    private func extractData(ptr: UnsafeMutableRawPointer) -> [Data] {
        let alignedPtr = ptr;
        let dataArray = DataArray.fromRawPtr(alignedPtr)
        let arr = dataArray.arr
        return arr
    }
    
}

//@_cdecl("swift_data_create")
//func dataArray(size: UInt64) ->  {
//    
//}

