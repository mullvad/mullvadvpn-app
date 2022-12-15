//
//  ChangeLogNotifierUIHandler.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-12-15.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol ChangeLogNotifierUIHandler {
    func showVersionChanges(_ changes: [String])
}
