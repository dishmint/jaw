import SwiftUI

// The container app exists so macOS has somewhere to register the `jaw` UTI and
// load the Quick Look extension. It shows a short explainer window; the real work
// happens in JAWQuickLookExtension.
@main
struct JAWQuickLookApp: App {
    var body: some Scene {
        WindowGroup("JAW Quick Look") {
            ContentView()
        }
    }
}

struct ContentView: View {
    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            Text("JAW Quick Look")
                .font(.system(size: 22, weight: .bold))

            Text("This app registers the .jaw file type with macOS and provides a "
                 + "syntax-highlighted Quick Look preview.")
                .fixedSize(horizontal: false, vertical: true)

            Divider()

            VStack(alignment: .leading, spacing: 8) {
                Label("Keep this app in /Applications.", systemImage: "folder")
                Label("Select a .jaw file in Finder and press Space.", systemImage: "space")
                Label("You can quit this window — the preview keeps working.",
                      systemImage: "checkmark.seal")
            }
            .font(.callout)
        }
        .padding(28)
        .frame(width: 420)
    }
}
