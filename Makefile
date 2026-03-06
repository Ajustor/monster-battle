# ──────────────────────────────────────────────────────────────────
# Monster Battle — Makefile
# ──────────────────────────────────────────────────────────────────

# ── Variables ────────────────────────────────────────────────────
PKG            := monster-battle-android
ARCH           := arm64
PLATFORM       := android
TARGET         := aarch64-linux-android
ANDROID_PKG    := com.ajustor.monsterbattle

AAB            := target/x/release/android/$(PKG).aab
APK_UNSIGNED   := target/x/release/android/$(PKG)-unsigned.apk
APK_ALIGNED    := target/x/release/android/$(PKG)-aligned.apk
APK            := target/x/release/android/$(PKG).apk

BUILD_TOOLS    := $(HOME)/Android/Sdk/build-tools/36.1.0
ZIPALIGN       := $(BUILD_TOOLS)/zipalign
APKSIGNER      := $(BUILD_TOOLS)/apksigner
AAPT2          := $(BUILD_TOOLS)/aapt2
ANDROID_JAR    := $(HOME)/Android/Sdk/platforms/android-36/android.jar
RES_DIR        := crates/android/res
KEYSTORE       := target/x/release/android/debug.keystore
KEYSTORE_PASS  := android

# ── Desktop ──────────────────────────────────────────────────────

.PHONY: check build run

## Vérifier la compilation desktop
check:
	cargo check -p $(PKG)

## Builder en mode debug (desktop)
build:
	cargo build -p $(PKG)

## Lancer en mode debug (desktop)
run:
	cargo run -p $(PKG)

# ── Android ──────────────────────────────────────────────────────

.PHONY: android-check android-build android-apk android-install android-deploy android-run android-clean

## Vérifier la compilation Android
android-check:
	cargo check -p $(PKG) --target $(TARGET)

## Builder l'AAB Android (release)
android-build:
	x build -p $(PKG) --release --platform $(PLATFORM) --arch $(ARCH)

## Générer un keystore debug s'il n'existe pas
$(KEYSTORE):
	@mkdir -p $(dir $(KEYSTORE))
	keytool -genkeypair \
		-keystore $(KEYSTORE) \
		-storepass $(KEYSTORE_PASS) \
		-alias androiddebugkey \
		-keypass $(KEYSTORE_PASS) \
		-keyalg RSA -keysize 2048 -validity 10000 \
		-dname "CN=Debug,O=Debug,C=US"

## Signer et zipaligner l'APK (avec icône)
android-apk: android-build $(KEYSTORE)
	@# xbuild produit un APK nommé .aab — on le copie d'abord
	cp $(AAB) $(APK_UNSIGNED)
	@# Compiler les ressources (icônes) via aapt2
	@rm -rf target/x/release/android/compiled_res target/x/release/android/res-overlay.apk target/x/release/android/res-tmp
	@mkdir -p target/x/release/android/compiled_res
	$(AAPT2) compile --dir $(RES_DIR) -o target/x/release/android/compiled_res/
	$(AAPT2) link \
		-I $(ANDROID_JAR) \
		--manifest crates/android/AndroidManifest.xml \
		-o target/x/release/android/res-overlay.apk \
		target/x/release/android/compiled_res/*.flat
	@# Extraire les ressources compilées et les injecter dans l'APK
	@mkdir -p target/x/release/android/res-tmp
	cd target/x/release/android/res-tmp && unzip -o ../res-overlay.apk
	python3 scripts/inject_resources.py target/x/release/android/$(notdir $(APK_UNSIGNED)) target/x/release/android/res-tmp
	@rm -rf target/x/release/android/compiled_res target/x/release/android/res-overlay.apk target/x/release/android/res-tmp
	@# Zipalign (alignement 4 bytes, requis par Android)
	$(ZIPALIGN) -f -p 4 $(APK_UNSIGNED) $(APK_ALIGNED)
	@# Signer avec apksigner
	$(APKSIGNER) sign \
		--ks $(KEYSTORE) \
		--ks-pass pass:$(KEYSTORE_PASS) \
		--ks-key-alias androiddebugkey \
		--key-pass pass:$(KEYSTORE_PASS) \
		--out $(APK) \
		$(APK_ALIGNED)
	@# Nettoyage des fichiers intermédiaires
	rm -f $(APK_UNSIGNED) $(APK_ALIGNED)
	@echo ""
	@echo "✅ APK signé : $(APK)"
	@echo "   Taille : $$(du -h $(APK) | cut -f1)"

## Installer l'APK sur un appareil connecté
android-install: android-apk
	@adb devices | grep -q 'device$$' || (echo "❌ Aucun appareil détecté. Active le débogage USB et branche ton téléphone." && exit 1)
	adb install -r $(APK)
	@echo "✅ APK installé ! Lance l'app sur ton téléphone."

## Installer + lancer + afficher les logs
android-deploy: android-install
	adb shell am start -n $(ANDROID_PKG)/android.app.NativeActivity
	adb logcat -s MonsterBattle:* bevy:* wgpu:*

## Builder + installer + lancer sur un appareil connecté (avec icône)
android-run: android-install
	adb shell am start -n $(ANDROID_PKG)/android.app.NativeActivity

## Nettoyer les artefacts Android
android-clean:
	rm -rf target/x
	rm -f $(KEYSTORE)

# ── TUI ──────────────────────────────────────────────────────────

.PHONY: tui tui-release

## Lancer le jeu TUI (terminal)
tui:
	cargo run -p monster-battle-tui

## Lancer le jeu TUI en release
tui-release:
	cargo run -p monster-battle-tui --release

# ── Général ──────────────────────────────────────────────────────

.PHONY: clean help

## Nettoyer tout
clean:
	cargo clean
	rm -rf target/x

## Afficher l'aide
help:
	@echo "Monster Battle — Commandes disponibles :"
	@echo ""
	@echo "  Desktop :"
	@echo "    make check          — Vérifier la compilation desktop"
	@echo "    make build          — Builder (debug, desktop)"
	@echo "    make run            — Lancer (debug, desktop)"
	@echo ""
	@echo "  Android :"
	@echo "    make android-check  — Vérifier la compilation Android"
	@echo "    make android-build  — Builder l'AAB Android (release)"
	@echo "    make android-apk    — Convertir l'AAB en APK universel"
	@echo "    make android-install — Installer l'APK sur l'appareil"
	@echo "    make android-deploy — Installer + lancer + logs (adb)"
	@echo "    make android-run    — Builder + installer + lancer sur l'appareil"
	@echo "    make android-clean  — Nettoyer les artefacts Android"
	@echo ""
	@echo "  TUI :"
	@echo "    make tui            — Lancer le jeu TUI (terminal)"
	@echo "    make tui-release    — Lancer le jeu TUI (release)"
	@echo ""
	@echo "  Général :"
	@echo "    make clean          — Nettoyer tout"
	@echo "    make help           — Afficher cette aide"
