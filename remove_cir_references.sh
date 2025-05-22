#!/bin/bash
# Script to remove CIR references and replace them with CI

set -e

YELLOW='\033[1;33m'
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Starting removal of CIR references...${NC}"

# Change to the CI directory
cd "$(dirname "$0")"

# Stage 1: Create backups of files we're going to modify
echo -e "${YELLOW}Creating backups...${NC}"
mkdir -p ./backup_before_cir_removal
find . -type f -name "*.rs" -o -name "*.md" -o -name "*.toml" | while read -r file; do
  cp "$file" "./backup_before_cir_removal/$(basename "$file").bak"
done
echo -e "${GREEN}Backups created in ./backup_before_cir_removal/${NC}"

# Stage 2: Update Rust files - handle specific keyword replacements
echo -e "${YELLOW}Updating Rust files...${NC}"
find . -type f -name "*.rs" | while read -r file; do
  # Replace CIR with CI but not inside strings and comments
  sed -i '' 's/\bCIR\b/CI/g' "$file"
  
  # Replace CIR_ environment variables with CI_
  sed -i '' 's/CIR_/CI_/g' "$file"
  
  # Replace cir with ci (for binary/command names)
  sed -i '' 's/\bcir\b/ci/g' "$file"
  
  echo "  Processed $file"
done

# Stage 3: Update markdown files
echo -e "${YELLOW}Updating markdown files...${NC}"
find . -type f -name "*.md" | while read -r file; do
  # Replace CIR with CI
  sed -i '' 's/\bCIR\b/CI/g' "$file"
  
  # Replace _CIR. directives with _CI.
  sed -i '' 's/_CIR\./_CI\./g' "$file"
  
  # Replace references to CIR tool with CI tool
  sed -i '' 's/\bcir\b/ci/g' "$file"
  
  echo "  Processed $file"
done

# Stage 4: Update TOML files
echo -e "${YELLOW}Updating TOML files...${NC}"
find . -type f -name "*.toml" | while read -r file; do
  # Replace references to CIR with CI
  sed -i '' 's/\bCIR\b/CI/g' "$file"
  sed -i '' 's/\bcir\b/ci/g' "$file"
  
  echo "  Processed $file"
done

# Stage 5: Check for any leftover references
echo -e "${YELLOW}Checking for any leftover CIR references...${NC}"
LEFTOVER_REFS=$(grep -r "CIR" --include="*.rs" --include="*.md" --include="*.toml" . | wc -l)
if [ "$LEFTOVER_REFS" -gt 0 ]; then
  echo -e "${RED}Found $LEFTOVER_REFS remaining references to CIR. Manual review needed.${NC}"
  echo -e "${YELLOW}Locations:${NC}"
  grep -r "CIR" --include="*.rs" --include="*.md" --include="*.toml" .
else
  echo -e "${GREEN}No remaining references to CIR found!${NC}"
fi

echo -e "${GREEN}CIR reference removal complete!${NC}"
echo -e "${YELLOW}You should now rebuild the project with 'cargo build' and test it.${NC}"