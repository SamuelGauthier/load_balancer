GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

wait_for_server() {
    local name=$1
    local port=$2

    echo "Waiting for $name to be available on port $port..."

    while ! nc -z localhost $port; do
        sleep 1
    done

    echo "$name is up and running on port $port!"
}

kill_pids() {
    for pid in "$@"; do
        if kill -0 "$pid" 2>/dev/null; then
            echo "Killing process with PID: $pid"
            kill "$pid"

            sleep 1
            if kill -0 "$pid" 2>/dev/null; then
                echo "Process $pid is still running!"
            else
                echo "Process $pid has been successfully killed."
            fi
        else
            echo "Process with PID $pid is not running."
        fi
    done
}
