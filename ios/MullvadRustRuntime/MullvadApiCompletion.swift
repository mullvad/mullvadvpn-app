//
//  MullvadApiCompletion.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-01-16.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

@_silgen_name("mullvad_api_completion_finish")
func mullvadApiCompletionFinish(
    response: SwiftMullvadApiResponse,
    completionCookie: UnsafeMutableRawPointer
) {
    let completionBridge = Unmanaged<MullvadApiCompletion>
        .fromOpaque(completionCookie)
        .takeRetainedValue()
    let apiResponse = MullvadApiResponse(response: response)

    completionBridge.completion(apiResponse)
}

public class MullvadApiCompletion {
    public var completion: (MullvadApiResponse) -> Void

    public init(completion: @escaping ((MullvadApiResponse) -> Void)) {
        self.completion = completion
    }
}

@_silgen_name("connection_mode_provider_initial")
func ConnectionModeProviderInitial(rawPointer: UnsafeMutableRawPointer) {
    let accessMethodIterator = Unmanaged<SwiftConnectionModeProvider>
        .fromOpaque(rawPointer)
        .takeRetainedValue()
    accessMethodIterator.initial()
}
