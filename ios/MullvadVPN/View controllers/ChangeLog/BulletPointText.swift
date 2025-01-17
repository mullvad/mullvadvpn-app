//
//  BulletPointText.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-01-10.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct BulletPointText: View {
    let text: String
    let bullet = "•"

    var body: some View {
        HStack(alignment: .firstTextBaseline) {
            Text(bullet)
                .font(.body)
                .foregroundColor(UIColor.secondaryTextColor.color)
            Text(text)
                .font(.body)
                .foregroundColor(UIColor.secondaryTextColor.color)
                .lineLimit(nil)
                .multilineTextAlignment(.leading)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}
