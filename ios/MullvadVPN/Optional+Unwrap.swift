
//
//  Optional+Unwrap.swift
//  MullvadVPN
//
//  Created by pronebird on 02/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation

class NilUnwrapError: Error {}

extension Optional {
    func unwrap() throws -> Wrapped {
        switch self {
        case .some(let value):
            return value
        case .none:
            throw NilUnwrapError()
        }
    }
}
