//
//  AsyncExample.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-16.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

@_silgen_name("completion_finish")
func finishCompletion(
    response: SwiftMullvadApiResponse,
    completionCookie: UnsafeMutableRawPointer
) {
    let completionBridge = Unmanaged<CompletionBridge>
        .fromOpaque(completionCookie)
        .takeUnretainedValue()
    let apiResponse = MullvadApiResponse(response: response)

    completionBridge.completion(apiResponse)
}

public class CompletionBridge {
    public var completion: (MullvadApiResponse) -> Void

    public init(completion: @escaping ((MullvadApiResponse) -> Void)) {
        self.completion = completion
    }
}

// @_silgen_name("async_finish")
// func finishAsyncExample(
//    response: SwiftMullvadApiResponse,
//    asyncCookie: UnsafeMutableRawPointer
// ) {
//    let completion = Unmanaged<AsyncCompletionBridge>
//        .fromOpaque(asyncCookie)
//        .takeUnretainedValue()
//    let apiResponse = MullvadApiResponse(response: response)
//
//    completion.continuation.resume(returning: apiResponse)
// }

// public class AsyncCompletionBridge {
//    public let continuation: CheckedContinuation<MullvadApiResponse, Never>
//
//    public init(continuation: CheckedContinuation<MullvadApiResponse, Never>) {
//        self.continuation = continuation
//    }
// }
