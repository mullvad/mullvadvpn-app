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

    var body: some View {
        TimelineView(.animation) { timeline in
            let elapsed = timeline.date.timeIntervalSince(startDate)

            Rectangle()
                .fill(.white)
                .colorEffect(
                    ShaderLibrary.fireBorder(
                        .float2(Float(chipSize.width), Float(chipSize.height)),
                        .float2(Float(chipSize.width), Float(chipSize.height)),
                        .float(Float(elapsed)),
                        .float(Float(cornerRadius))
                    )
                )
        }
        .allowsHitTesting(false)
    }
}

// MARK: - Lens flare burst (triggered alongside 3D spin)

struct LensFlareView: View {
    let isVisible: Bool

    @State private var flareScale: CGFloat = 0
    @State private var flareOpacity: Double = 0

    var body: some View {
        ZStack {
            // Horizontal streak
            Capsule()
                .fill(
                    LinearGradient(
                        colors: [.clear, .white.opacity(0.9), .cyan.opacity(0.6), .clear],
                        startPoint: .leading,
                        endPoint: .trailing
                    )
                )
                .frame(width: 80, height: 4)

            // Vertical streak (cross)
            Capsule()
                .fill(
                    LinearGradient(
                        colors: [.clear, .white.opacity(0.7), .blue.opacity(0.4), .clear],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                )
                .frame(width: 3, height: 50)

            // Central glow
            Circle()
                .fill(
                    RadialGradient(
                        colors: [.white, .cyan.opacity(0.6), .blue.opacity(0.2), .clear],
                        center: .center,
                        startRadius: 0,
                        endRadius: 20
                    )
                )
                .frame(width: 40, height: 40)

            // Hexagonal bokeh ring
            ForEach(0..<6, id: \.self) { i in
                Circle()
                    .fill(Color.cyan.opacity(0.15))
                    .frame(width: 6, height: 6)
                    .offset(x: 18 * cos(Double(i) * .pi / 3), y: 18 * sin(Double(i) * .pi / 3))
            }
        }
        .scaleEffect(flareScale)
        .opacity(flareOpacity)
        .allowsHitTesting(false)
        .onChange(of: isVisible) {
            if isVisible {
                triggerFlare()
            }
        }
    }

    private func triggerFlare() {
        flareScale = 0.2
        flareOpacity = 0
        withAnimation(.easeOut(duration: 0.15)) {
            flareScale = 1.3
            flareOpacity = 1.0
        }
        withAnimation(.easeIn(duration: 0.35).delay(0.15)) {
            flareScale = 0.5
            flareOpacity = 0
        }
    }

    private func cos(_ v: Double) -> CGFloat { CGFloat(Foundation.cos(v)) }
    private func sin(_ v: Double) -> CGFloat { CGFloat(Foundation.sin(v)) }
}

// MARK: - Composite overlay: fire/rainbow + chromatic aberration + shake + flare + 3D spin

struct GotaTunChipOverlay: ViewModifier {
    @State private var rotation: Double = 0
    @State private var isSpinning = false
    @State private var chipSize: CGSize = .zero
    @State private var timer: Timer?
    @State private var startDate = Date.now
    @State private var showFlare = false

    // Screen shake
    @State private var shakeOffset: CGSize = .zero
    @State private var isShaking = false

    /// Decided once at creation: which border effect to show.
    private enum BorderEffect: CaseIterable {
        case rainbow, fire, chromaticAberration, liquidGlass
    }

    private let borderEffect: BorderEffect = {
        switch Int.random(in: 0...3) {
        case 0: .rainbow
        case 1: .fire
        case 2: .chromaticAberration
        default: .liquidGlass
        }
    }()

    /// Apparent z-depth of the chip during rotation, in points.
    private let chipDepth: CGFloat = 12

    func body(content: Content) -> some View {
        TimelineView(.animation) { timeline in
            let elapsed = timeline.date.timeIntervalSince(startDate)

            content
                .sizeOfView { chipSize = $0 }
                .overlay(
                    Group {
                        switch borderEffect {
                        case .rainbow:
                            RainbowShimmerBorder(cornerRadius: 8, lineWidth: 2)
                        case .fire:
                            FireBorderView(chipSize: chipSize, cornerRadius: 8)
                        case .chromaticAberration, .liquidGlass:
                            EmptyView()
                        }
                    }
                )
                // Layer effects: chromatic aberration or liquid glass
                .if(borderEffect == .chromaticAberration) { view in
                    view.layerEffect(
                        ShaderLibrary.chromaticAberration(
                            .float(Float(elapsed)),
                            .float(2.0)
                        ),
                        maxSampleOffset: CGSize(width: 4, height: 4)
                    )
                }
                .if(borderEffect == .liquidGlass) { view in
                    view.layerEffect(
                        ShaderLibrary.liquidGlass(
                            .float2(Float(chipSize.width), Float(chipSize.height)),
                            .float(Float(elapsed)),
                            .float(8.0)
                        ),
                        maxSampleOffset: CGSize(width: 4, height: 4)
                    )
                }
                .overlay(LensFlareView(isVisible: showFlare))
                .offset(shakeOffset)
                // Thick "slab" effect
                .background(
                    RoundedRectangle(cornerRadius: 8)
                        .fill(Color.black.opacity(0.6))
                        .frame(height: chipDepth)
                        .offset(y: chipDepth / 2)
                        .opacity(slabOpacity)
                )
                .rotation3DEffect(
                    .degrees(rotation),
                    axis: (x: 0, y: 1, z: 0),
                    anchor: .center,
                    anchorZ: 0,
                    perspective: 0.4
                )
        }
        .onAppear { startTickTimer() }
        .onDisappear { timer?.invalidate() }
    }

    /// Fires once per second; each tick rolls for spin (1%) and shake (2%).
    private func startTickTimer() {
        timer = Timer.scheduledTimer(withTimeInterval: 1, repeats: true) { _ in
            let roll = Int.random(in: 0..<100)
            if roll == 0, !isSpinning {
                triggerSpin()
            }
            if roll >= 1 && roll <= 2, !isShaking {
                triggerShake()
            }
        }
    }

    private func triggerSpin() {
        isSpinning = true
        showFlare = true
        withAnimation(.easeInOut(duration: 0.45)) {
            rotation += 360
        } completion: {
            isSpinning = false
            showFlare = false
        }
    }

    private func triggerShake() {
        isShaking = true
        // Rapid jitter for 0.3 seconds (10 frames at ~0.03s each)
        Task { @MainActor in
            for _ in 0..<10 {
                withAnimation(.linear(duration: 0.03)) {
                    shakeOffset = CGSize(
                        width: CGFloat.random(in: -3...3),
                        height: CGFloat.random(in: -3...3)
                    )
                }
                try? await Task.sleep(for: .milliseconds(30))
            }
            withAnimation(.easeOut(duration: 0.05)) {
                shakeOffset = .zero
            }
            isShaking = false
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
