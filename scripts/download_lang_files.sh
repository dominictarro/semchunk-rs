#!/bin/sh

source $(dirname "${BASH_SOURCE[0]}")/env.sh

function display_help() {
    echo "Usage: $0 [options]"
    echo
    echo A script to download language files for tokenizer models.
    echo
    echo "Options:"
    echo "  -h, --help          Show this help message and exit"
    echo "  -v, --verbose       Enable verbose mode"
    echo "  -d, --dir DIR       Directory to download files to. Will create if it doesn't exist. Defaults to '.'"
    echo
    echo "Examples:"
    echo "  $0 -v -d ./test/data"
    echo "  $0 --verbose --dir ./test/data"
    echo
}

# Check for help flag
if check_flag "-h" "--help" "$@"; then
    display_help
    exit 0
fi

VERBOSE=`check_flag "-v" "--verbose" "$@" && echo "true" || echo "false"`
DIR=`get_param_value "-d" "--dir" "$@"`
if ! [ -d "$DIR" ]; then
    mkdir -p $DIR
fi

function echo_verbose() {
    if [ "$VERBOSE" == "true" ]; then
        echo "$1"
    fi
}

function download_if_not_exists() {
    path=$DIR/$1
    if ! [ -f "$path" ]; then
        echo_verbose "Downloading $path from $2"
        silent_flag=`[ "$VERBOSE" == "true" ] && echo "" || echo "-s"`
        curl $2 -o $path $silent_flag
        return $?
    else
        echo_verbose "$path already exists"
    fi
}

results=0
download_if_not_exists "roberta-base-vocab.json" "https://s3.amazonaws.com/models.huggingface.co/bert/roberta-base-vocab.json"
results=$((results+$?))
download_if_not_exists "roberta-base-merges.txt" "https://s3.amazonaws.com/models.huggingface.co/bert/roberta-base-merges.txt"
results=$((results+$?))

exit $([ $results -eq 0 ] && echo 0 || echo 1)
