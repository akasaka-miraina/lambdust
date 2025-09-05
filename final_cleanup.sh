#!/bin/bash

# Disable files that import SIMD functionality
FILES_TO_DISABLE=(
    "src/stdlib/srfi133_vectors.rs"
    "src/stdlib/vectors.rs"
)

# Add #![cfg(feature = "never-enabled")] to each file
for file in "${FILES_TO_DISABLE[@]}"; do
    if [ -f "$file" ]; then
        echo "Disabling $file"
        sed -i '' '1s/^/#![cfg(feature = "never-enabled")]\n/' "$file"
    fi
done

echo "Disabled $(wc -w <<< "${FILES_TO_DISABLE[@]}") additional files"
