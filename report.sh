#!/bin/bash

# Check if a directory was provided as an argument
if [ -z "$1" ]; then
  echo "Usage: $0 DIRECTORY"
  exit 1
fi

# Find all .sii files in the directory
FILES=$(ls "$1"/*.sii 2>/dev/null)

# Loop through each .sii file and compare its size to other .sii files with the same stem
for FILE in $FILES; do
  # Get the file stem by removing the extension
  STEM=$(basename "$FILE" | cut -d'.' -f1)

  # Check if there are other .png files with the same stem
  OTHER_FILES=$(ls "$1/$STEM"*.png 2>/dev/null)
  if [ -n "$OTHER_FILES" ]; then
    # Loop through each other .png file and compare its size to the current .sii file
    for OTHER_FILE in $OTHER_FILES; do
      if [ "$FILE" != "$OTHER_FILE" ]; then
        SIZE1=$(wc -c <"$FILE")
        SIZE2=$(wc -c <"$OTHER_FILE")
        DIFF=$(expr $SIZE1 - $SIZE2)
        PERCENT=$(echo "scale=2; $DIFF / $SIZE1 * 100" | bc)
        echo "$FILE: $SIZE1 bytes; $OTHER_FILE: $SIZE2 bytes; Difference: $DIFF bytes ($PERCENT%)"
      fi
    done
  fi
done
