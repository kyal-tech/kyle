#!/bin/bash
set -o pipefail

RESULTS_FILE="/tmp/opencode/example_test_results.txt"
rm -f "$RESULTS_FILE"
touch "$RESULTS_FILE"

KYC="./target/debug/ky"
EXAMPLES="examples"

declare -A COMPILE_RESULT
declare -A RUN_RESULT
declare -A RUN_OUTPUT

for f in "$EXAMPLES"/*.ky; do
  BASENAME=$(basename "$f" .ky)
  echo "=== TESTING: $f ===" | tee -a "$RESULTS_FILE"

  # Step 1: Compile
  COMPILE_OUTPUT=$($KYC build "$f" 2>&1)
  COMPILE_EXIT=$?
  if [ $COMPILE_EXIT -ne 0 ]; then
    echo "  COMPILE FAILED (exit=$COMPILE_EXIT)" | tee -a "$RESULTS_FILE"
    echo "  OUTPUT: $COMPILE_OUTPUT" | tee -a "$RESULTS_FILE"
    COMPILE_RESULT["$BASENAME"]="FAIL"
    RUN_RESULT["$BASENAME"]="SKIP"
    continue
  fi
  COMPILE_RESULT["$BASENAME"]="OK"
  echo "  COMPILE OK" >> "$RESULTS_FILE"

  # Step 2: Check if binary was produced and has main (try running it)
  BINARY="examples/$BASENAME"
  if [ -f "$BINARY" ] && [ -x "$BINARY" ]; then
    # Check if binary has main symbol (rough check by running it)
    RUN_OUTPUT=$($BINARY 2>&1)
    RUN_EXIT=$?
    RUN_OUTPUT["$BASENAME"]="$RUN_OUTPUT"
    if [ $RUN_EXIT -ne 0 ]; then
      echo "  RUN FAILED (exit=$RUN_EXIT)" | tee -a "$RESULTS_FILE"
      echo "  OUTPUT: $RUN_OUTPUT" | tee -a "$RESULTS_FILE"
      RUN_RESULT["$BASENAME"]="FAIL"
    else
      echo "  RUN OK (exit=0)" >> "$RESULTS_FILE"
      RUN_RESULT["$BASENAME"]="OK"
    fi
  else
    echo "  (no binary produced or not executable)" >> "$RESULTS_FILE"
    RUN_RESULT["$BASENAME"]="N/A"
  fi

  # Clean up binary if it exists
  [ -f "$BINARY" ] && rm -f "$BINARY"
  # Clean up .o file
  [ -f "examples/$BASENAME.o" ] && rm -f "examples/$BASENAME.o"
  echo "" >> "$RESULTS_FILE"
done

# Summary
echo ""
echo "=============================================="
echo "SUMMARY"
echo "=============================================="
COMPILE_OK=0
COMPILE_FAIL=0
RUN_OK=0
RUN_FAIL=0
RUN_NA=0
for key in "${!COMPILE_RESULT[@]}"; do
  if [ "${COMPILE_RESULT[$key]}" = "OK" ]; then
    ((COMPILE_OK++))
  else
    ((COMPILE_FAIL++))
  fi
  if [ "${RUN_RESULT[$key]}" = "OK" ]; then
    ((RUN_OK++))
  elif [ "${RUN_RESULT[$key]}" = "FAIL" ]; then
    ((RUN_FAIL++))
  elif [ "${RUN_RESULT[$key]}" = "N/A" ]; then
    ((RUN_NA++))
  fi
done

echo "Compile OK: $COMPILE_OK"
echo "Compile FAIL: $COMPILE_FAIL"
echo "Run OK: $RUN_OK"
echo "Run FAIL: $RUN_FAIL"
echo "Run N/A (no binary/no main): $RUN_NA"
echo "=============================================="

echo ""
echo "--- COMPILE FAILURES ---"
for key in "${!COMPILE_RESULT[@]}"; do
  if [ "${COMPILE_RESULT[$key]}" != "OK" ]; then
    echo "  $key.ky"
  fi
done

echo ""
echo "--- RUN FAILURES ---"
for key in "${!RUN_RESULT[@]}"; do
  if [ "${RUN_RESULT[$key]}" = "FAIL" ]; then
    echo "  $key.ky"
    echo "    Output: ${RUN_OUTPUT[$key]}"
  fi
done

echo ""
echo "--- FILES THAT COMPILED OK ---"
for key in "${!COMPILE_RESULT[@]}"; do
  if [ "${COMPILE_RESULT[$key]}" = "OK" ]; then
    echo "  $key.ky"
  fi
done

echo ""
echo "--- FILES THAT RAN OK ---"
for key in "${!RUN_RESULT[@]}"; do
  if [ "${RUN_RESULT[$key]}" = "OK" ]; then
    echo "  $key.ky"
  fi
done
