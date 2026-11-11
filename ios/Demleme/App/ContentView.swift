import SwiftUI

struct ContentView: View {
    var body: some View {
        ZStack {
            DemlemeColor.bg.ignoresSafeArea()
            Text("Demleme")
                .font(.largeTitle.bold())
                .foregroundStyle(DemlemeColor.label)
                .accessibilityIdentifier("app.title")
        }
    }
}

#Preview {
    ContentView()
}
