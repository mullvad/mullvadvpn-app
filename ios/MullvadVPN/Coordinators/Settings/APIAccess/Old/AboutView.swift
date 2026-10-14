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
                        .foregroundStyle(Color.mullvadTextPrimary)
                        .padding(.bottom, 16.0)
                        .multilineTextAlignment(.center)
                }

                if let preamble {
                    Text(preamble)
                        .font(.mullvadSmall)
                        .foregroundStyle(Color.mullvadTextPrimary)
                        .padding(.bottom, 16.0)
                        .multilineTextAlignment(.center)
                }

                ForEach(Array(paragraphs.enumerated()), id: \.offset) { _, paragraph in
                    Text(paragraph)
                        .font(.mullvadTiny)
                        .foregroundStyle(Color.secondaryTextColor)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .padding(.bottom, 15)
                }
            }
            .padding(.horizontal, 16.0)
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
