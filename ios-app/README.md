# Monster Battle — iOS App

## Prérequis

- macOS avec Xcode installé
- Rust avec les targets iOS : `rustup target add aarch64-apple-ios x86_64-apple-ios`

## Builder la lib Rust

```bash
# Device (iPhone/iPad physique)
cargo build -p monster-battle-ios --target aarch64-apple-ios --release

# Simulateur x86_64
cargo build -p monster-battle-ios --target x86_64-apple-ios --release

# Universal (device + simulateur)
lipo -create \
  target/aarch64-apple-ios/release/libmonster_battle_ios.a \
  target/x86_64-apple-ios/release/libmonster_battle_ios.a \
  -output ios-app/libmonster_battle_ios.a
```

## Intégration Xcode

1. Ouvrir (ou créer) un projet Xcode vide (SwiftUI App ou Single View App)
2. **File → Add Files** → sélectionner `libmonster_battle_ios.a`
3. Dans **Build Settings → Other Linker Flags** ajouter :
   ```
   -lmonster_battle_ios -lc++ -framework UIKit -framework Metal -framework QuartzCore -framework AVFoundation
   ```
4. Dans **Build Settings → Library Search Paths** ajouter :
   ```
   $(PROJECT_DIR)
   ```

## Appeler le point d'entrée depuis Swift

Dans `AppDelegate.swift` :

```swift
// Déclare la fonction Rust
@_silgen_name("ios_main")
func iosMain()

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {
    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        // Lance le moteur Bevy (bloquant)
        iosMain()
        return true
    }
}
```

## Permissions (Info.plist)

Ajouter selon les besoins :
```xml
<!-- Réseau (PvP) -->
<key>NSAllowsArbitraryLoads</key>
<true/>

<!-- Accès documents -->
<key>UIFileSharingEnabled</key>
<true/>
```

## CI GitHub Actions

Le workflow `.github/workflows/ios.yml` build automatiquement la lib universelle
sur chaque push vers `feat/v1` ou `main`.
