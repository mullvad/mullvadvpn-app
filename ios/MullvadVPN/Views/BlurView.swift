//
//  BlurView.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-12-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

/// Blurred (background) view using a `UIBlurEffect`.
struct BlurView: View {
    var style: UIBlurEffect.Style

    var body: some View {
        VisualEffectView(effect: UIBlurEffect(style: style))
            .opacity(0.8)
    }
}
