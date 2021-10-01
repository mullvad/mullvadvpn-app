//
//  Cancellable.swift
//  MullvadVPN
//
//  Created by pronebird on 23/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol Cancellable {
    func cancel()
}
