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
