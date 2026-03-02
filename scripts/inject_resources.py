#!/usr/bin/env python3
"""Inject aapt2-compiled resources + AndroidManifest.xml into xbuild's APK."""

import sys
import zipfile
import os
import shutil


def main():
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <apk_path> <overlay_dir>", file=sys.stderr)
        sys.exit(1)

    apk_src = sys.argv[1]
    tmp = sys.argv[2]

    apk_new = apk_src + ".new"

    src = zipfile.ZipFile(apk_src, "r")
    dst = zipfile.ZipFile(apk_new, "w", zipfile.ZIP_DEFLATED)

    # Collect all resource files from the overlay
    res_files = set()
    res_dir = os.path.join(tmp, "res")
    if os.path.isdir(res_dir):
        for dp, _, fns in os.walk(res_dir):
            for f in fns:
                res_files.add(os.path.relpath(os.path.join(dp, f), tmp))

    # Files we want to replace from the overlay
    overlay_files = {"resources.arsc", "AndroidManifest.xml"} | res_files

    # Copy everything from original APK EXCEPT the overlay files
    for item in src.infolist():
        if item.filename not in overlay_files:
            # resources.arsc from the original must also be stored uncompressed
            dst.writestr(item, src.read(item.filename))

    # Write the overlay's AndroidManifest.xml (has android:icon reference)
    manifest_path = os.path.join(tmp, "AndroidManifest.xml")
    if not os.path.exists(manifest_path):
        print(f"❌ Erreur : {manifest_path} introuvable dans l'overlay", file=sys.stderr)
        src.close()
        dst.close()
        os.remove(apk_new)
        sys.exit(1)
    dst.write(manifest_path, "AndroidManifest.xml")

    # Write the overlay's resources.arsc — MUST be stored uncompressed
    # and 4-byte aligned (required by Android R+ / API 30+)
    arsc_path = os.path.join(tmp, "resources.arsc")
    if not os.path.exists(arsc_path):
        print(f"❌ Erreur : {arsc_path} introuvable dans l'overlay", file=sys.stderr)
        src.close()
        dst.close()
        os.remove(apk_new)
        sys.exit(1)
    info = zipfile.ZipInfo("resources.arsc")
    info.compress_type = zipfile.ZIP_STORED  # uncompressed
    with open(arsc_path, "rb") as f:
        dst.writestr(info, f.read())

    # Write all resource files (mipmap icons, etc.)
    for rf in res_files:
        dst.write(os.path.join(tmp, rf), rf)

    src.close()
    dst.close()

    shutil.move(apk_new, apk_src)
    print("  📦 Ressources + Manifest injectés dans APK")


if __name__ == "__main__":
    main()
