//
//  RustProblemReportRequest.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadRustRuntimeProxy

final public class RustProblemReportRequest {
    private let rustPointer: UnsafeMutablePointer<SwiftProblemReportRequest>

    public init(from request: REST.ProblemReportRequest) {
        rustPointer = UnsafeMutablePointer<SwiftProblemReportRequest>.allocate(capacity: 1)

        let addressPointer = request.address.toUnsafePointer()
        let messagePointer = request.message.toUnsafePointer()
        let logPointer = request.log.toUnsafePointer()

        let metadataPointer = swift_map_new()

        for (key, value) in request.metadata {
            key.withCString { keyPtr in
                value.withCString { valuePtr in
                    swift_map_add(metadataPointer, keyPtr, valuePtr)
                }
            }
        }

        rustPointer.initialize(to: SwiftProblemReportRequest(
            address: addressPointer,
            address_len: UInt(request.address.utf8.count),
            message: messagePointer,
            message_len: UInt(request.message.utf8.count),
            log: logPointer,
            log_len: UInt(request.log.utf8.count),
            meta_data: metadataPointer
        ))
    }

    public func getPointer() -> UnsafePointer<SwiftProblemReportRequest> {
        return UnsafePointer(rustPointer)
    }

    public func release() {
        let metadataPointer = rustPointer.pointee.meta_data
        swift_map_free(metadataPointer)

        rustPointer.deinitialize(count: 1)
        rustPointer.deallocate()
    }
}
