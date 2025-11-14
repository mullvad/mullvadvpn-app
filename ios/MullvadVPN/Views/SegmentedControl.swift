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
    @State private var id: UUID = .init()
    @Namespace var animation
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
                        .padding(5)
                        .font(.mullvadTinySemiBold)
                        .foregroundStyle(.white)
                        .frame(maxWidth: .infinity)  // Makes the text take all the available space
                        .contentShape(Rectangle())  // Makes the tappable area extend beyond just the text
                        .background(
                            Group {
                                if segment == selectedSegment {
                                    Capsule()
                                        .fill(UIColor.SegmentedControl.selectedColor.color)
                                        .matchedGeometryEffect(id: id, in: animation)
                                } else {
                                    Capsule()
                                        .fill(.clear)
                                }
                            }
                        )
                        .frame(maxWidth: .infinity)
                }
                .disabled(segment == selectedSegment)
            }
        }
        .padding(4)
        .background {
            Capsule(style: .circular)
                .fill(UIColor.SegmentedControl.backgroundColor.color)
        }
        .frame(minHeight: 44)
    }
}

#Preview {
    @Previewable @State var selectedSegment = "Exit"
    VStack {
        Spacer()
        SegmentedControl(
            segments: ["Entry", "Exit"],
            selectedSegment: $selectedSegment
        )
        Spacer()
    }
    .background(Color.mullvadBackground)
}
