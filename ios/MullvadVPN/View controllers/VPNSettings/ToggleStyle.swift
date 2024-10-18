//
//  ToggleStyle.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-10-18.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct RedToggleStyle: ToggleStyle {
    let width: CGFloat = 50

    func makeBody(configuration: Self.Configuration) -> some View {
        HStack {
            configuration.label
            Spacer()

            ZStack(alignment: configuration.isOn ? .trailing : .leading) {
                RoundedRectangle(cornerRadius: 20)
                    .frame(width: width, height: width / 2)
                    .foregroundColor(.clear)
                    .overlay(
                        RoundedRectangle(cornerRadius: 25)
                            .stroke(.white, lineWidth: 2)
                    )

                RoundedRectangle(cornerRadius: 20)
                    .frame(width: (width / 2) - 4, height: width / 2 - 6)
                    .padding(4)
                    .foregroundColor(configuration.isOn ? .green : .red)
                    .onTapGesture {
                        withAnimation {
                            configuration.$isOn.wrappedValue.toggle()
                        }
                    }
            }
        }
    }
}

private struct ContainerView: View {
    @State var isOn: Bool
    var body: some View {
        Toggle("hello", isOn: $isOn)
    }
}

#Preview {
    ContainerView(isOn: false).toggleStyle(RedToggleStyle())
        .preferredColorScheme(.dark)
}
