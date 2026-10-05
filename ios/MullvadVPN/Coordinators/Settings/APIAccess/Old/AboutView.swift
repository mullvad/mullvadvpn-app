import SwiftUI

struct AboutView: View {
    let header: String?
    let preamble: String?
    let paragraphs: [AttributedString]

    var body: some View {
        ScrollView {
            VStack(spacing: 0) {
                if let header {
                    Text(header)
                        .font(.mullvadLarge)
                        .padding(.bottom, 32)
                }

                if let preamble {
                    Text(preamble)
                        .font(.mullvadSmall)
                        .padding(.bottom, 24)
                }

                ForEach(Array(paragraphs.enumerated()), id: \.offset) { _, paragraph in
                    Text(paragraph)
                        .font(.mullvadTiny)
                        .foregroundStyle(Color.secondaryTextColor)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.bottom, 15)
                }
            }
            .padding(UIMetrics.contentInsets.toEdgeInsets)
            .foregroundStyle(Color.mullvadTextPrimary)
        }
        .background(Color.mullvadBackground)
    }
}

#Preview {
    AboutView(
        header: "Known issues",
        preamble: "These are some known issues",
        paragraphs: [
            AttributedString("iOS features known to be affected:"),
            AttributedString("AirDrop, AirPlay, Screen Mirroring"),
        ]
    )
}
