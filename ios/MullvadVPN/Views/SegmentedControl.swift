//
//  SegmentedControl.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2025-09-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

class SegmentedControlViewModel: ObservableObject {
    @Published var selectedSegmentIndex = 0
}

struct SegmentedControl<Segment: StringProtocol>: View {
    var segments: [Segment]
    @ObservedObject var viewModel: SegmentedControlViewModel
    public var onSelectedSegment: ((Int) -> Void)?

    func isSelected(segment: Segment) -> Bool {
        viewModel.selectedSegmentIndex == segments.firstIndex(of: segment)
    }

    var body: some View {
        GeometryReader { proxy in
            HStack(spacing: 0) {
                ForEach(segments, id: \.self) { segment in
                    // The segments are expected to be already localised
                    Text(segment)
                        .font(.mullvadSmallSemiBold)
                        .foregroundStyle(.white)
                        .frame(maxWidth: .infinity) // Makes the text take all the available space
                        .contentShape(Rectangle()) // Makes the tappable area extend beyond just the text
                        .onTapGesture {
                            withAnimation(.easeInOut(duration: 0.25)) {
                                viewModel.selectedSegmentIndex = segments.firstIndex(of: segment)!
                                onSelectedSegment?(viewModel.selectedSegmentIndex)
                            }
                        }
                        .background(
                            Group {
                                if isSelected(segment: segment) {
                                    Capsule()
                                        .fill(UIColor.SegmentedControl.selectedColor.color)
                                        .frame(height: 36)
                                } else {
                                    Capsule()
                                        .fill(UIColor.SegmentedControl.backgroundColor.color)
                                        .frame(height: 36)
                                }
                            }
                        )
                }
            }
            .padding([.leading, .trailing], 4) // Insets the inner shape to not overlay with the outer one
            .frame(maxWidth: .infinity, maxHeight: proxy.size.height)
            .background(
                Capsule(style: .circular)
                    .fill(UIColor.SegmentedControl.backgroundColor.color)
            )
            .clipShape(Capsule())
        }
    }
}

#Preview {
    VStack {
        Spacer()
        SegmentedControl(
            segments: ["Entry", "Exit"],
            viewModel: SegmentedControlViewModel(),
            onSelectedSegment: { newIndex in print("Selected \(newIndex)") }
        )
        .frame(height: 44)
        Spacer()
    }
    .background(Color.mullvadBackground)
}
