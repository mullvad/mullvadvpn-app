//
//  AlertInteractor.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-08-23.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging

final class AlertInteractor {
    private let logger = Logger(label: "AlertInteractor")

    var presentation: AlertPresentation

    init(presentation: AlertPresentation) {
        self.presentation = presentation
    }
}
