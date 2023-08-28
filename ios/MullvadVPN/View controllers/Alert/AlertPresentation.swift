//
//  AlertPresentation.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-08-23.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct AlertPresentation: Identifiable {
    let id = ObjectIdentifier(NSUUID())

    var header: String?
    var icon: AlertIcon?
    var title: String?
    let message: String?
    let buttons: [AlertAction]
}

extension AlertPresentation: Equatable, Hashable {
    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: AlertPresentation, rhs: AlertPresentation) -> Bool {
        lhs.id == rhs.id
    }
}
