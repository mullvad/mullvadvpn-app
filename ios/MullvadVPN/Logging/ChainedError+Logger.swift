//
//  ChainedError+Logger.swift
//  MullvadVPN
//
//  Created by pronebird on 02/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

extension Logger {
    func error<T: ChainedError>(
        chainedError: T,
        message: @autoclosure () -> String? = nil,
        metadata: @autoclosure () -> Logger.Metadata? = nil,
        source: @autoclosure () -> String? = nil,
        file: String = #file,
        function: String = #function,
        line: UInt = #line
    )
    {
        log(
            level: .error,
            Message(
                stringLiteral: chainedError.displayChain(message: message())
            ),
            metadata: metadata(),
            source: source(),
            file: file,
            function: function,
            line: line
        )
    }
}
