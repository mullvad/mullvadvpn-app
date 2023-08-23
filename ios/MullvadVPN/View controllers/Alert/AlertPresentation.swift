//
//  AlertPresentation.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-08-23.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

struct AlertPresentation {
    var style: UIAlertController.Style
    var header: String?
    var title: String?
    let message: String?
    let buttons: [AlertAction]
    var onDismiss: ((AlertAction) -> Void)?
}

extension AlertPresentation: Equatable, Hashable {
    func hash(into hasher: inout Hasher) {
        hasher.combine(title)
    }

    static func == (lhs: AlertPresentation, rhs: AlertPresentation) -> Bool {
        lhs.title == rhs.title
    }
}
