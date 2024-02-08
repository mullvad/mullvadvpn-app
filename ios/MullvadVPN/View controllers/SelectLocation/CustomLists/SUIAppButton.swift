//
//  SUIAppButton.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-09.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import SwiftUI

/// SwiftUI wrapper for ``AppButton``.
struct SUIAppButton: UIViewRepresentable {
    let text: String
    let style: AppButton.Style

    func makeUIView(context: Context) -> AppButton {
        let button = AppButton(style: style)
        button.setTitle(text, for: .normal)

        return button
    }

    func updateUIView(_ appButton: AppButton, context: Context) {}
}
