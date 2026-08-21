//
//  InputViewAppearance.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-08-05.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct InputViewAppearance {
    var titleFont: Font = .mullvadTinySemiBold
    var font: Font = .mullvadSmall
    var foregroundColor: Color = .MullvadTextField.textInput
    var placeholderColor: Color = .MullvadTextField.inputPlaceholder
    var clearButtonImage: Image = .mullvadIconCross
    var cornerRadius: CGFloat = 4.0
    var messageFont: Font = .mullvadTiny
    var backgroundColor: Color = .MullvadTextField.background
    var height: CGFloat
    var spacing: CGFloat = 4.0
}
