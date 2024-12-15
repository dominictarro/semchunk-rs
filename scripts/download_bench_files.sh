#!/bin/bash

source $(dirname "${BASH_SOURCE[0]}")/env.sh

function display_help() {
    echo "Usage: $0 [options]"
    echo
    echo A script to download corpus files for benchmarking.
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
DIR=$(echo "$DIR" | sed 's:/*$::')
if ! [ -d "$DIR" ]; then
    mkdir -p $DIR
fi

function echo_verbose() {
    if [ "$VERBOSE" == "true" ]; then
        echo "$1"
    fi
}


zip_fn=$DIR/gutenberg.zip
if ! [ -f "$zip_fn" ]; then
    echo_verbose "Downloading $zip_fn from https://raw.githubusercontent.com/nltk/nltk_data/gh-pages/packages/corpora/gutenberg.zip"
    silent_flag=`[ "$VERBOSE" == "true" ] && echo "" || echo "-s"`
    curl https://raw.githubusercontent.com/nltk/nltk_data/gh-pages/packages/corpora/gutenberg.zip -o $zip_fn $silent_flag
else
    echo_verbose "$zip_fn already exists"
fi
quiet_flag=`[ "$VERBOSE" == "true" ] && echo "" || echo "-qq"`
unzip -u $quiet_flag -d $DIR $zip_fn
