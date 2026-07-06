//
//  StaticButtonStyle.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-06-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct StaticButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
    }
}

struct InactiveButtonStyle: ButtonStyle {
    let isInactive: Bool = true

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .opacity(isInactive ? 0.4 : (configuration.isPressed ? 0.7 : 1.0))
    }
}
