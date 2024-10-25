//
//  ToggleStyle.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-10-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct RedToggleStyle: ToggleStyle {
    let width: CGFloat = 57
    let height: CGFloat = 35
    let circleRadius: CGFloat = 28
    let action: (() -> Void)?

    func makeBody(configuration: Self.Configuration) -> some View {
        HStack {
            configuration.label

            if let action {
                Button(action: action) {
                    Image(.iconInfo)
                }
                .tint(.white)
            }
            Spacer()

            ZStack(alignment: configuration.isOn ? .trailing : .leading) {
                Capsule(style: .circular)
                    .frame(width: width, height: height)
                    .foregroundColor(.clear)
                    .overlay(
                        RoundedRectangle(cornerRadius: 20)
                            .stroke(
                                Color(uiColor: UIColor(white: 1.0, alpha: 0.8)),
                                lineWidth: 2
                            )
                    )

                Circle()
                    .frame(width: circleRadius, height: circleRadius)
                    .padding(4)
                    .foregroundColor(configuration.isOn ? .green : .red)
                    .onTapGesture {
                        withAnimation {
                            configuration.$isOn.wrappedValue.toggle()
                        }
                    }
            }
        }
        .padding(4)
    }
}

private struct ContainerView: View {
    @State var isOn: Bool
    var body: some View {
        Toggle("Malware", isOn: $isOn)
    }
}

#Preview {
    VStack {
        ContainerView(isOn: false).toggleStyle(RedToggleStyle(action: {}))
            .preferredColorScheme(.dark)
        ContainerView(isOn: false).toggleStyle(RedToggleStyle(action: nil))
            .preferredColorScheme(.dark)
    }
}
