//
//  RoundedContainerModifier.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-07-24.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//
import SwiftUI

struct RoundedCornerModifier: ViewModifier {
    var cornerRadius: CGFloat = .infinity
    var corners: UIRectCorner = .allCorners
    var insertBy: CGFloat = 0

    var borderColor: Color = .clear
    var borderWidth: CGFloat = 0

    func body(content: Content) -> some View {
        content
            .clipShape(
                RoundedCorner(
                    cornerRadius: cornerRadius,
                    corners: corners,
                    insertBy: insertBy
                )
            )
            .overlay {
                RoundedCorner(
                    cornerRadius: cornerRadius,
                    corners: corners,
                    insertBy: insertBy
                )
                .stroke(borderColor, lineWidth: borderWidth)
            }
    }
}

private struct RoundedCorner: Shape {
    var cornerRadius: CGFloat = .infinity
    var corners: UIRectCorner = .allCorners
    var insertBy: CGFloat = 0

    func path(in rect: CGRect) -> Path {
        let insetRect = rect.insetBy(dx: insertBy, dy: insertBy)
        let path = UIBezierPath(
            roundedRect: insetRect,
            byRoundingCorners: corners,
            cornerRadii: CGSize(width: cornerRadius, height: cornerRadius)
        )
        return Path(path.cgPath)
    }
}
