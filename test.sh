#!/bin/bash

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Build the project
echo "Building pure-ts..."
cargo build --release
if [ $? -ne 0 ]; then
    echo -e "${RED}Build failed${NC}"
    exit 1
fi

BINARY="./target/release/pure-ts"
PASS_COUNT=0
FAIL_COUNT=0

# Function to run test and check for expected errors
run_test() {
    local test_file=$1
    local test_name=$2
    shift 2
    local expected_errors=("$@")
    
    echo -e "\n${YELLOW}Testing: $test_name${NC}"
    echo "File: $test_file"
    
    # Check if file exists
    if [ ! -f "$test_file" ]; then
        echo -e "${YELLOW}SKIP${NC} - Test file not found"
        return
    fi
    
    # Run the linter and capture output
    output=$($BINARY "$test_file" 2>&1)
    exit_code=$?
    
    # Check if it should pass or fail
    if [ ${#expected_errors[@]} -eq 0 ]; then
        # Should pass (no errors expected)
        if [ $exit_code -eq 0 ]; then
            echo -e "${GREEN}✓ PASS${NC} - No errors as expected"
            ((PASS_COUNT++))
        else
            echo -e "${RED}✗ FAIL${NC} - Unexpected errors found:"
            echo "$output"
            ((FAIL_COUNT++))
        fi
    else
        # Should fail with specific errors
        if [ $exit_code -ne 0 ]; then
            all_found=true
            for error in "${expected_errors[@]}"; do
                if ! echo "$output" | grep -q "$error"; then
                    echo -e "${RED}✗ FAIL${NC} - Expected error not found: $error"
                    all_found=false
                    ((FAIL_COUNT++))
                    break
                fi
            done
            
            if $all_found; then
                echo -e "${GREEN}✓ PASS${NC} - All expected errors found"
                ((PASS_COUNT++))
            else
                echo "Actual output:"
                echo "$output" | grep -E "\[.*\]"
            fi
        else
            echo -e "${RED}✗ FAIL${NC} - Expected errors but none found"
            ((FAIL_COUNT++))
        fi
    fi
}

# Test no-classes rule
run_test "test_files/test_class.ts" "no-classes rule" \
    "no-classes"

# Test no-enums rule
run_test "test_files/test_enum.ts" "no-enums rule" \
    "no-enums"

# Test no-namespace-imports rule
run_test "test_files/test_namespace_import.ts" "no-namespace-imports rule" \
    "no-namespace-imports"

# Test no-member-assignments rule
run_test "test_files/test_member_assignment.ts" "no-member-assignments rule" \
    "no-member-assignments"

# Test import extensions rule
run_test "test_files/test_import_extensions.ts" "import-extensions rule" \
    "missing-ts-extension"

# Test no-getters-setters rule
run_test "test_files/test_getters_setters.ts" "no-getters-setters rule" \
    "no-getters" \
    "no-setters"

# Test must-use-return-value rule
run_test "test_files/test_return_value.ts" "must-use-return-value rule" \
    "must-use-return-value"

# Test no-delete rule
run_test "test_files/test_delete.ts" "no-delete rule" \
    "no-delete"

# Test no-this-in-functions rule
run_test "test_files/test_this.ts" "no-this-in-functions rule" \
    "no-this-in-functions"

# Test no-throw rule
run_test "test_files/test_throw.ts" "no-throw rule" \
    "no-throw" \
    "no-try-catch"

# Test no-foreach rule
run_test "test_files/test_foreach.ts" "no-foreach rule" \
    "no-foreach"

# Test no-filename-dirname rule
run_test "test_files/test_filename_dirname.ts" "no-filename-dirname rule" \
    "no-filename" \
    "no-dirname"

# Test interface-extends-only rule
run_test "test_files/test_interface.ts" "interface-extends-only rule" \
    "interface-extends-only"

# Test no-eval-function rule
run_test "test_files/test_eval.ts" "no-eval-function rule" \
    "no-eval" \
    "no-new-function"

# Test no-object-assign rule
run_test "test_files/test_object_assign.ts" "no-object-assign rule" \
    "no-object-assign"

# Test no-constant-condition rule
run_test "test_files/test_constant_condition.ts" "no-constant-condition rule" \
    "no-constant-condition"

# Test switch-case-block rule
run_test "test_files/test_switch_case_block.ts" "switch-case-block rule" \
    "switch-case-block"

# Test no-as-cast rule
run_test "test_files/test_as_cast.ts" "no-as-cast rule" \
    "no-as-cast" \
    "no-as-upcast" \
    "no-type-assertion"

# Test let-requires-type rule
run_test "test_files/test_let_requires_type.ts" "let-requires-type rule" \
    "let-requires-type"

# Test catch-error-handling rule
run_test "test_files/test_catch_error.ts" "catch-error-handling rule" \
    "no-try-catch" \
    "try-must-return-ok" \
    "catch-must-return-err" \
    "catch-error-handling"

# Test jsdoc-param-match rule
run_test "test_files/test_jsdoc_params.ts" "jsdoc-param-match rule" \
    "jsdoc-param-missing" \
    "jsdoc-param-unknown" \
    "jsdoc-param-count" \
    "param-missing-type"

# Test no-top-level-side-effects rule
run_test "test_files/test_top_level_calls.ts" "no-top-level-side-effects rule" \
    "no-top-level-side-effects"

# Test no-named-exports rule
run_test "test_files/test_exports.ts" "no-named-exports rule" \
    "no-named-exports"

# Test export-const-needs-type rule
run_test "test_files/test_export_variables.ts" "export-const-needs-type rule" \
    "no-export-let" \
    "export-const-needs-type"

# Summary
echo -e "\n========================================="
echo -e "Test Results:"
echo -e "${GREEN}PASSED: $PASS_COUNT${NC}"
echo -e "${RED}FAILED: $FAIL_COUNT${NC}"
echo -e "========================================="

if [ $FAIL_COUNT -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed${NC}"
    exit 1
fi