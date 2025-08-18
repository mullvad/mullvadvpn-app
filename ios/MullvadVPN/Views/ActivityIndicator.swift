//
//  ActivityIndicator.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-08-18.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

// This is a native SwiftUI replacement for SpinnerActivityIndicatorView

import SwiftUI

struct ActivityIndicator: View {
    @State private var rotationAngle = 0.0

    var body: some View {
        ZStack {
            Circle()
                .stroke(
                    lineWidth: 4
                )
                .opacity(0.25)
            Circle()
                .trim(from: 0.25, to: 1.0)
                .rotation(.degrees(rotationAngle))
                .stroke(
                    style: StrokeStyle(lineWidth: 4, lineCap: .round)
                )
            Path()
        }
        .onAppear {
            withAnimation(.linear(duration: 0.5).repeatForever(autoreverses: false)) {
                rotationAngle = 360.0
            }
        }
        .onDisappear {
            rotationAngle = 0.0
        }
    }
}

#Preview {
    ActivityIndicator().frame(width: 20, height: 20)
}
