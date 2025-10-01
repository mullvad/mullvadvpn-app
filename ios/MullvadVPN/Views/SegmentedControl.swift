//
//  SegmentedControl.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2025-09-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct SegmentedControl<Segment>: View where Segment: CustomStringConvertible, Segment: Hashable {
    let segments: [Segment]
    @Binding var selectedSegment: Segment

    func isSelected(segment: Segment) -> Bool {
        segment == selectedSegment
    }

    var body: some View {
        HStack(spacing: 0) {
            ForEach(segments, id: \.self) { segment in
                // The segments are expected to be already localised
                Button {
                    withAnimation {
                        selectedSegment = segment
                    }
                } label: {
                    Text(LocalizedStringKey(segment.description))
                        .padding()
                        .font(.mullvadSmallSemiBold)
                        .foregroundStyle(.white)
                        .frame(maxWidth: .infinity) // Makes the text take all the available space
                        .contentShape(Rectangle()) // Makes the tappable area extend beyond just the text
                        .background(
                            Group {
                                if isSelected(segment: segment) {
                                    Capsule()
                                        .fill(UIColor.SegmentedControl.selectedColor.color)
                                        .id("selected")
                                } else {
                                    Capsule()
                                        .fill(.clear)
                                }
                            }
                        )
                        .padding(4)
                        .frame(maxWidth: .infinity)
                }
                .disabled(isSelected(segment: segment))
            }
        }
        .background(
            Capsule(style: .circular)
                .fill(UIColor.SegmentedControl.backgroundColor.color)
        )
    }
}

@available(iOS 17, *)
#Preview {
    @Previewable @State var selectedSegment = "Exit"
    VStack {
        Spacer()
        SegmentedControl(
            segments: ["Entry", "Exit"],
            selectedSegment: $selectedSegment
        )
        .frame(height: 44)
        Spacer()
    }
    .background(Color.mullvadBackground)
}
