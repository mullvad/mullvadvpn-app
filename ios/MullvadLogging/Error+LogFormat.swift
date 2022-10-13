//
//  Error+LogFormat.swift
//  MullvadLogging
//
//  Created by pronebird on 26/09/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension Error {
    public func logFormatError() -> String {
        let nsError = self as NSError
        var message = ""

        let description = (self as? CustomErrorDescriptionProtocol)?
            .customErrorDescription ?? localizedDescription

        message += "\(description) (domain = \(nsError.domain), code = \(nsError.code))"

        return message
    }
}
