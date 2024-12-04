//
//  FeatureIndicatorsScrollContainerView.swift
//  MullvadVPN
//
//  Created by Marco Nikic on 2024-12-12.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import SwiftUI

struct FeatureIndicatorsScrollContainerView<ContentView: View>: View {
    var isExpanded: Binding<Bool>
    @ViewBuilder
    let content: ContentView

    var body: some View {
        ScrollView {
            content
                .frame(maxWidth: .infinity)
                .background(.blue)
        }
        .frame(maxHeight: isExpanded.wrappedValue ? .infinity : 40)
    }
}

#Preview {
    ExampleView().background(UIColor.secondaryColor.color)
}

private struct ExampleView: View {
    @State var isExpanded = false

    var body: some View {
        VStack {
            Button(action: {
                isExpanded.toggle()
            }, label: {
                Text("Toggle layout")
            })
            FeatureIndicatorsScrollContainerView(isExpanded: $isExpanded) {
                if isExpanded {
                    BigLayoutView()
                } else {
                    SmallLayoutView()
                }
            }
        }
    }
}

private struct SmallLayoutView: View {
    var body: some View {
        HStack {
            ForEach(0 ..< 3) { index in
                Text("hehehjr \(index)")
            }
        }
    }
}

private struct BigLayoutView: View {
    var body: some View {
        Group {
            VStack {
                ForEach(0 ..< 5) { _ in
                    HStack {
                        ForEach(0 ..< 8) { index in
                            Text("hello \(index)")
                        }
                    }
                }
            }
        }
    }
}
