import SwiftUI

struct MullvadProgressViewStyle: ProgressViewStyle {
    @State var isAnimating = false
    func makeBody(configuration: Configuration) -> some View {
        Image.mullvadIconSpinner
            .resizable()
            .frame(maxWidth: 48, maxHeight: 48)
            .rotationEffect(.degrees(isAnimating ? 360 : 0))
            .onAppear {
                withAnimation(
                    .linear(duration: 0.6).repeatForever(autoreverses: false)
                ) {
                    isAnimating = true
                }
            }
    }
}

#Preview {
    ProgressView()
        .progressViewStyle(MullvadProgressViewStyle())
        .background(Color.mullvadBackground)
}
