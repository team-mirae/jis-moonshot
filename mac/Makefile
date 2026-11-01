# Builds JISMoonshot.app (universal arm64 + x86_64) without Xcode: Command Line Tools are enough.
# JIS Moonshot is one build (docs/moonshot.md): predictions always on, the English modes and JIS
# Kana always in.
#   make                 build build/full/JISMoonshot.app
#   make LEGACY_PREDICT=1  build build/full-legacy/JISMoonshot.app: predictions ranked by the rules
#                        alone (spec §10.3), without the learned ranker (§10.13) or its model
#   make install         copy it to ~/Library/Input Methods, register it (enabling JIS Moonshot,
#                        JIS Kana and JIS English on a first install), restart it, and add a login
#                        agent that starts it at login
#   make dist            build/<variant>/JISMoonshot-<version>.zip for a release

# 1 ranks predictions by the rules alone (engine `legacy-predict` feature, spec §10.13); 0 (the
# default) adds the learned ranker and its model, predict/learned.bin.
LEGACY_PREDICT ?= 0
ifeq ($(LEGACY_PREDICT),1)
  VARIANT        := full-legacy
  CARGO_FEATURES := --features legacy-predict
  DIST_SUFFIX    := -legacy
else
  VARIANT        := full
  CARGO_FEATURES :=
  DIST_SUFFIX    :=
endif
# Always: predictions (spec §10) and the English modes (spec §3.6.2). The Swift flags stay so the
# sources read as they did in JIS Korean.
SWIFT_DEFINES  := -D PREDICT -D ENGLISH

ENGINE     := ../engine
HEADER     := $(ENGINE)/include/jis_korean.h
BUILD      := build/$(VARIANT)
APP        := $(BUILD)/JISMoonshot.app
TARGET_DIR := $(ENGINE)/target/mac-$(VARIANT)
INSTALL_TO := $(HOME)/Library/Input Methods
LOGIN_AGENT := $(HOME)/Library/LaunchAgents/me.sjun.inputmethod.JISMoonshot.login.plist
MIN_MACOS  := 13.0
SOURCES    := $(wildcard Sources/*.swift)
CARGO      := $(shell command -v cargo || echo $(HOME)/.cargo/bin/cargo)

# Code signing: your Apple Development certificate if the keychain has one (sign in to Xcode → Settings
# → Accounts, then Manage Certificates), else ad hoc. The permission macOS grants (Accessibility,
# for the system-wide 英数 / かな keys) follows the signature: with a certificate
# they survive rebuilds; ad hoc, every build is a new app to macOS and has to be granted again.
# Override with `make SIGN_IDENTITY="..."`.
SIGN_IDENTITY ?= $(or $(shell security find-identity -v -p codesigning 2>/dev/null | sed -n 's/.*"\(Apple Development: [^"]*\)".*/\1/p' | head -1),-)

LSREGISTER := /System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister

export MACOSX_DEPLOYMENT_TARGET := $(MIN_MACOS)

.PHONY: all engine install uninstall dist clean

all: $(APP)

# Spotlight would register build copies with LaunchServices (see install); keep it out of build/.
$(shell mkdir -p build && touch build/.metadata_never_index)

# --- engine: Rust static library, one per arch, merged with lipo

$(BUILD)/libjis_korean.a: $(wildcard $(ENGINE)/src/*.rs) $(ENGINE)/Cargo.toml
	@mkdir -p $(BUILD)
	$(CARGO) build --manifest-path $(ENGINE)/Cargo.toml --release $(CARGO_FEATURES) --target-dir $(TARGET_DIR) --target aarch64-apple-darwin
	$(CARGO) build --manifest-path $(ENGINE)/Cargo.toml --release $(CARGO_FEATURES) --target-dir $(TARGET_DIR) --target x86_64-apple-darwin
	lipo -create -output $@ \
		$(TARGET_DIR)/aarch64-apple-darwin/release/libjis_korean.a \
		$(TARGET_DIR)/x86_64-apple-darwin/release/libjis_korean.a

engine: $(BUILD)/libjis_korean.a

# --- Swift: one binary per arch, merged with lipo

$(BUILD)/%/JISMoonshot: $(SOURCES) $(HEADER) $(BUILD)/libjis_korean.a
	@mkdir -p $(dir $@)
	swiftc -O -swift-version 5 $(SWIFT_DEFINES) -target $*-apple-macos$(MIN_MACOS) \
		-import-objc-header $(HEADER) \
		-framework Cocoa -framework InputMethodKit -framework Carbon \
		$(SOURCES) $(BUILD)/libjis_korean.a -o $@

$(BUILD)/JISMoonshot: $(BUILD)/arm64/JISMoonshot $(BUILD)/x86_64/JISMoonshot
	lipo -create -output $@ $^

# --- icons: the logo and one badge per mode, drawn in docs/logo (make_logo.py, render.sh) and
#     committed as 16 px and 32 px PNGs; each pair becomes one .tiff

ICONS := $(BUILD)/moonshot.tiff $(BUILD)/mode-a.tiff $(BUILD)/mode-a-kana.tiff $(BUILD)/mode-a-qwerty.tiff \
         $(BUILD)/mode-a-dvorak.tiff $(BUILD)/mode-a-colemak.tiff

$(BUILD)/%.tiff: Resources/icons/%.png Resources/icons/%@2x.png
	@mkdir -p $(BUILD)
	tiffutil -cathidpicheck Resources/icons/$*.png Resources/icons/$*@2x.png -out $@

# --- Keyboard Viewer layouts, one per mode: its legends on ABC (JIS), from the engine's layout
#     tables. Mode n's legends are legends-n.tsv. JIS Kana uses Mode A's: it consumes every key it
#     maps, so the layout only types what it passes on.

$(BUILD)/legends-%.tsv: $(wildcard $(ENGINE)/src/*.rs) $(ENGINE)/examples/legends.rs
	@mkdir -p $(BUILD)
	$(CARGO) run -q --manifest-path $(ENGINE)/Cargo.toml $(CARGO_FEATURES) --example legends --target-dir $(TARGET_DIR) -- $* > $@

$(BUILD)/make-keylayout: tools/make-keylayout/main.swift Sources/KeyMap.swift $(HEADER)
	@mkdir -p $(BUILD)
	swiftc -O -import-objc-header $(HEADER) Sources/KeyMap.swift tools/make-keylayout/main.swift -o $@

$(BUILD)/JISMoonshot.keylayout: $(BUILD)/make-keylayout $(BUILD)/legends-0.tsv
	$(BUILD)/make-keylayout $(BUILD)/legends-0.tsv $@ JISMoonshot JIS -27101

# The English modes: their symbols on a JIS keyboard, and capitals under Caps Lock. JIS English (3),
# JIS Dvorak (7) and JIS Colemak (8). make-keylayout takes ⌥, ⌘ and Ctrl from ABC, so shortcuts stay
# on QWERTY in Dvorak and Colemak too (spec §3.6.4). Ids are -271xx (the mode id in the last two digits), away from JIS Korean's -234xx.
$(BUILD)/JISMoonshotEnglish.keylayout: $(BUILD)/make-keylayout $(BUILD)/legends-3.tsv
	$(BUILD)/make-keylayout $(BUILD)/legends-3.tsv $@ JISMoonshotEnglish JIS -27103 caps

$(BUILD)/JISMoonshotDvorak.keylayout: $(BUILD)/make-keylayout $(BUILD)/legends-7.tsv
	$(BUILD)/make-keylayout $(BUILD)/legends-7.tsv $@ JISMoonshotDvorak JIS -27107 caps

$(BUILD)/JISMoonshotColemak.keylayout: $(BUILD)/make-keylayout $(BUILD)/legends-8.tsv
	$(BUILD)/make-keylayout $(BUILD)/legends-8.tsv $@ JISMoonshotColemak JIS -27108 caps

KEYLAYOUTS := $(BUILD)/JISMoonshot.keylayout $(BUILD)/JISMoonshotEnglish.keylayout \
              $(BUILD)/JISMoonshotDvorak.keylayout $(BUILD)/JISMoonshotColemak.keylayout

# An earlier build installed the layout as its own bundle here; install removes it.
LAYOUTS_TO    := $(HOME)/Library/Keyboard Layouts

# --- bundle

PREDICT_FILES := $(ENGINE)/predict/ranker.bin $(ENGINE)/predict/common-en.txt $(ENGINE)/predict/common-ko.txt \
                 $(ENGINE)/predict/common-ja.txt $(ENGINE)/predict/kanji.bin $(ENGINE)/predict/LICENSE-mozc.txt \
                 $(ENGINE)/predict/english-nouns.txt $(ENGINE)/predict/LICENSE-wordnet.txt \
                 $(ENGINE)/predict/LICENSE-corpora.txt
ifneq ($(LEGACY_PREDICT),1)
  PREDICT_FILES += $(ENGINE)/predict/learned.bin
endif

$(APP): $(BUILD)/JISMoonshot $(ICONS) Info.plist $(KEYLAYOUTS) \
       Resources/JISMoonshotKeyboardLayouts-Info.plist Resources/en.lproj/InfoPlist.strings Resources/ko.lproj/InfoPlist.strings
	rm -rf $@
	mkdir -p $@/Contents/MacOS $@/Contents/Resources
	cp Info.plist $@/Contents/Info.plist
	cp $(BUILD)/JISMoonshot $@/Contents/MacOS/JISMoonshot
	cp $(ICONS) $@/Contents/Resources/
	cp $(KEYLAYOUTS) Resources/JISMoonshotKeyboardLayouts-Info.plist $@/Contents/Resources/
	@# JIS English, Dvorak and Colemak (spec §3.6.2, §8.2.1): English input modes of this input
	@# method, Roman script, so System Settings lists them under English and 英数 can select them.
	sh tools/add-english-mode.sh $@/Contents
	cp -R Resources/en.lproj Resources/ko.lproj $@/Contents/Resources/
	mkdir -p $@/Contents/Resources/predict
	cp $(PREDICT_FILES) $@/Contents/Resources/predict/
	codesign --force --sign "$(SIGN_IDENTITY)" $@

install: $(APP)
	@killall JISMoonshot 2>/dev/null || true
	mkdir -p "$(INSTALL_TO)"
	@# Swap the bundle in with renames: while the path is missing (an uninstall followed by a long
	@# rebuild), macOS drops the input source from running apps and re-registering doesn't bring it
	@# back (docs/troubleshooting/macos-input-method-not-launching.md).
	rm -rf "$(INSTALL_TO)/.JISMoonshot.app.new" "$(INSTALL_TO)/.JISMoonshot.app.old"
	ditto $(APP) "$(INSTALL_TO)/.JISMoonshot.app.new"
	if [ -d "$(INSTALL_TO)/JISMoonshot.app" ]; then mv "$(INSTALL_TO)/JISMoonshot.app" "$(INSTALL_TO)/.JISMoonshot.app.old"; fi
	mv "$(INSTALL_TO)/.JISMoonshot.app.new" "$(INSTALL_TO)/JISMoonshot.app"
	rm -rf "$(INSTALL_TO)/.JISMoonshot.app.old"
	rm -rf "$(LAYOUTS_TO)/JISMoonshot.bundle"
	@# Only the installed copy may be known to LaunchServices: input methods are launched by bundle
	@# id, and a build copy (or a deleted one) can be picked instead, so the input method never starts.
	@for app in build/full/JISMoonshot.app build/full-legacy/JISMoonshot.app; do \
		$(LSREGISTER) -u "$(CURDIR)/$$app" 2>/dev/null || true; done
	$(LSREGISTER) -f "$(INSTALL_TO)/JISMoonshot.app"
	swift tools/register.swift "$(INSTALL_TO)/JISMoonshot.app"
	@# Start it at login: the 英数 / かな tap lives in its process, which macOS otherwise starts only
	@# once JIS Moonshot is selected somewhere (tools/login-agent.plist).
	mkdir -p "$(dir $(LOGIN_AGENT))"
	cp tools/login-agent.plist "$(LOGIN_AGENT)"

# To reinstall, run install alone: uninstall first leaves the input source broken in running apps
# until it is removed and re-added in System Settings.
uninstall:
	@killall JISMoonshot 2>/dev/null || true
	rm -rf "$(INSTALL_TO)/JISMoonshot.app" "$(LAYOUTS_TO)/JISMoonshot.bundle" "$(LOGIN_AGENT)"

VERSION := $(shell plutil -extract CFBundleShortVersionString raw Info.plist)

ZIP := $(BUILD)/JISMoonshot-$(VERSION)$(DIST_SUFFIX).zip

dist: $(APP)
	rm -f $(ZIP)
	ditto -c -k --keepParent $(APP) $(ZIP)
	@echo $(ZIP)

clean:
	rm -rf build
