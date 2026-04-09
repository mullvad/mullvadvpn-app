//
//  RainbowShimmerModifier.swift
//  MullvadVPN
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

#if NEVER_IN_PRODUCTION
import SwiftUI

// MARK: - Rainbow outline stroke

struct RainbowShimmerBorder: View {
    @State private var phase: CGFloat = 0
    let cornerRadius: CGFloat
    let lineWidth: CGFloat

    var body: some View {
        RoundedRectangle(cornerRadius: cornerRadius)
            .stroke(
                AngularGradient(
                    colors: [.red, .orange, .yellow, .green, .cyan, .blue, .purple, .red],
                    center: .center,
                    angle: .degrees(phase * 360)
                ),
                lineWidth: lineWidth
            )
            .onAppear {
                withAnimation(.linear(duration: 2).repeatForever(autoreverses: false)) {
                    phase = 1
                }
            }
    }
}

// MARK: - Metal fire border applied as a layer effect

/// Renders procedural fire around the chip border using a Metal shader.
/// The view is larger than the chip to give flames room to extend outward.
struct FireBorderView: View {
    let chipSize: CGSize
    let cornerRadius: CGFloat
    @State private var startDate = Date.now

    /// Extra padding around the chip so flames have room to render outward.
    private let flameExtent: CGFloat = 16

    private var viewSize: CGSize {
        CGSize(
            width: chipSize.width + flameExtent * 2,
            height: chipSize.height + flameExtent * 2
        )
    }

    var body: some View {
        TimelineView(.animation) { timeline in
            let elapsed = timeline.date.timeIntervalSince(startDate)

            Rectangle()
                .fill(.white)
                .frame(width: viewSize.width, height: viewSize.height)
                .colorEffect(
                    ShaderLibrary.fireBorder(
                        .float2(Float(viewSize.width), Float(viewSize.height)),
                        .float2(Float(chipSize.width), Float(chipSize.height)),
                        .float(Float(elapsed)),
                        .float(Float(cornerRadius))
                    )
                )
        }
        .allowsHitTesting(false)
    }
}

// MARK: - Composite overlay: Metal fire + rainbow border + rare 3D spin

struct GotaTunChipOverlay: ViewModifier {
    @State private var rotation: Double = 0
    @State private var isSpinning = false
    @State private var chipSize: CGSize = .zero
    @State private var timer: Timer?

    /// Decided once at creation: `true` = rainbow shimmer, `false` = flames.
    private let showRainbow = Bool.random()

    /// Apparent z-depth of the chip during rotation, in points.
    private let chipDepth: CGFloat = 12

    func body(content: Content) -> some View {
        content
            .sizeOfView { chipSize = $0 }
            .overlay(
                Group {
                    if showRainbow {
                        RainbowShimmerBorder(cornerRadius: 8, lineWidth: 2)
                    } else {
                        FireBorderView(chipSize: chipSize, cornerRadius: 8)
                    }
                }
            )
            // Thick "slab" effect: a darkened copy offset in Z sits behind the chip
            .background(
                RoundedRectangle(cornerRadius: 8)
                    .fill(Color.black.opacity(0.6))
                    .frame(height: chipDepth)
                    .offset(y: chipDepth / 2)
                    // Only reveal the slab when the chip is turned sideways
                    .opacity(slabOpacity)
            )
            .rotation3DEffect(
                .degrees(rotation),
                axis: (x: 0, y: 1, z: 0),
                anchor: .center,
                anchorZ: 0,
                perspective: 0.4
            )
            .onAppear { startTickTimer() }
            .onDisappear { timer?.invalidate() }
    }

    /// Fires once per second; each tick has a 1% chance of triggering a spin.
    private func startTickTimer() {
        timer = Timer.scheduledTimer(withTimeInterval: 1, repeats: true) { _ in
            guard !isSpinning else { return }
            if Int.random(in: 0..<100) == 0 {
                triggerSpin()
            }
        }
    }

    private func triggerSpin() {
        isSpinning = true
        withAnimation(.easeInOut(duration: 0.45)) {
            rotation += 360
        } completion: {
            isSpinning = false
        }
    }

    /// Slab is most visible when chip is edge-on (90°/270°) and invisible when face-on.
    private var slabOpacity: Double {
        let angle = rotation.truncatingRemainder(dividingBy: 360)
        let radians = angle * .pi / 180
        return abs(sin(radians))
    }
}

extension View {
    func gotaTunStyle() -> some View {
        modifier(GotaTunChipOverlay())
    }
}
#endif
