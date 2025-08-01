# Localization Export Tool

Automate exporting localizable strings (XLIFF) from the MullvadVPN iOS project using `xcodebuild`.

This folder contains a Bash workflow that:

1. Builds the Xcode project (to emit/verify localized resources).
2. Exports localizations for one or more languages.
3. Logs each failed run (timestamped) to a git‑ignored `logs/` directory.
4. Uses a throwaway build output directory (`BUILD/`, also git‑ignored).
5. Cleans up Derived Data artifacts after export (configurable).

---

## Folder Structure

```
/mullvadvpn-app/ios/translation
├── locales
│   └── en.xliff
└── scripts
    ├── export_localizations.sh  # Main Bash script
    ├── build                    # Ephemeral DerivedData or build scratch dir (ignored)
    ├── logs                     # Timestamped run logs (ignored)
    └── README.md                # You're here

```

> **Important:** Ensure `build/` and `logs/` are ignored by Git (see [Git Ignore](#git-ignore)).

---

## Quick Start

```bash
cd Locale
chmod +x export-localizations.sh
./export-localizations.sh
```

By default the script uses values set near the top of the file (edit them before first run). You can override most settings via environment variables or CLI flags (see below).


## Multi‑Language Export

The script can loop languages. Set `EXPORT_LANGUAGES` to a comma‑separated list, e.g.:

```bash
EXPORT_LANGUAGES="da,de,en,es,fi,fr,it,ja,ko,my,nb,nl,pl,pt-PT,ru,sv,th,tr,zh-Hans,zh-Hant" ./export_localizations.sh
```

XLIFF output will be placed under:

```
locales/
  en.xliff
  sv.xliff
  de.xliff
  fr.xliff
```

(Actual filenames depend on what `xcodebuild -exportLocalizations` emits.)

---

## Logs

All stdout/stderr from each run is captured using `tee` into `logs/` with a timestamped filename, e.g.:

```
logs/
  build_20250723_142915.log
  build_20250724_094401.log
```

To prune logs older than 7 days automatically, the script includes an optional cleanup step (disabled by default; enable by setting `PRUNE_LOGS_DAYS`).

