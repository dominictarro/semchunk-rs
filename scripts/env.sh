function check_flag() {
    local short_flag="$1"
    local long_flag="$2"
    shift 2
    
    for param in "$@"; do
        if [[ "$param" == "$short_flag" || "$param" == "$long_flag" ]]; then
            return 0
        fi
    done
    
    return 1
}

function get_param_value() {
    local short_flag="$1"
    local long_flag="$2"
    shift 2

    while [[ $# -gt 0 ]]; do
        case "$1" in
            "$short_flag" | "$long_flag")
                if [[ -n "$2" && "$2" != -* ]]; then
                    echo "$2"
                    return 0
                else
                    echo "Error: Missing value for $1" >&2
                    return 1
                fi
                ;;
        esac
        shift
    done

    echo "."
    return 0
}

get_param_values() {
    local short_flag="$1"
    local long_flag="$2"
    shift 2
    local values=()

    while [[ $# -gt 0 ]]; do
        case "$1" in
            "$short_flag" | "$long_flag")
                if [[ -n "$2" && "$2" != -* ]]; then
                    values+=("$2")
                    shift 
                else
                    echo "Error: Missing value for $1" >&2
                    return 1
                fi
                ;;
        esac
        shift
    done

    echo "${values[@]}"
    return 0
}

export -f check_flag
export -f get_param_value
export -f get_param_values
