// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
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
