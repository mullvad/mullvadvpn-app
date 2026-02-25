#!/bin/bash

# Xcode 26 has a bug where screen recordings are left in the xcresult
# Staging directory instead of being committed to the Data store.
# The attachment metadata points to an error message instead of the video.
# This script recovers the video by copying it into the Data store
# and fixing the attachment metadata in the SQLite database.
#
# Usage: recover-screen-recordings.sh <path-to-xcresult>

set -eu

XCRESULT="${1:?Usage: $0 <path-to-xcresult>}"
STAGING_DIR="$XCRESULT/Staging"
DB="$XCRESULT/database.sqlite3"

if [ ! -d "$STAGING_DIR" ] || [ ! -f "$DB" ]; then
  echo "No Staging directory or database found, skipping"
  exit 0
fi

staging_videos=()
while IFS= read -r -d '' file; do
  if file "$file" | grep -q -i -E "MP4|MPEG|QuickTime|ISO Media"; then
    staging_videos+=("$file")
  fi
done < <(find "$STAGING_DIR" -type f -print0)

if [ ${#staging_videos[@]} -eq 0 ]; then
  echo "No video files found in Staging directory, skipping"
  exit 0
fi

echo "Found ${#staging_videos[@]} video(s) in Staging directory"

# Find broken screen recording attachments (UTI is public.text instead of a video type)
broken_refs=$(sqlite3 "$DB" \
  "SELECT xcResultKitPayloadRefId FROM Attachments
   WHERE name = 'kXCTAttachmentScreenRecording'
     AND uniformTypeIdentifier = 'public.text';")

if [ -z "$broken_refs" ]; then
  echo "No broken screen recording attachments found, skipping"
  exit 0
fi

broken_ref_array=()
while IFS= read -r ref; do
  broken_ref_array+=("$ref")
done <<< "$broken_refs"

echo "Found ${#broken_ref_array[@]} broken screen recording attachment(s)"

# Match staging videos to broken attachments by index
count=${#staging_videos[@]}
if [ ${#broken_ref_array[@]} -lt $count ]; then
  count=${#broken_ref_array[@]}
fi

for ((i=0; i<count; i++)); do
  video="${staging_videos[$i]}"
  ref="${broken_ref_array[$i]}"
  data_file="$XCRESULT/Data/data.${ref}"

  echo "Recovering screen recording $((i+1))/$count:"
  echo "  Video: $(basename "$video") ($(du -h "$video" | cut -f1))"
  echo "  Target: data.${ref}"

  # Move the video into the Data store, replacing the error text data file.
  # The data store normally uses zstd compression, but uncompressed files
  # are also accepted (the original error text was stored uncompressed).
  mv "$video" "$data_file"

  # Update the UTI in the database
  sqlite3 "$DB" \
    "UPDATE Attachments
     SET uniformTypeIdentifier = 'public.mpeg-4'
     WHERE xcResultKitPayloadRefId = '$ref'
       AND name = 'kXCTAttachmentScreenRecording'
       AND uniformTypeIdentifier = 'public.text';"
  echo "  Done - recovered successfully"
done

rmdir "$STAGING_DIR" 2>/dev/null || true
