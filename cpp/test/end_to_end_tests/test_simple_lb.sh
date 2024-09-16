#!/bin/bash

source ./common.sh

load_balancer_executable="../../build/lb"
urls_file="../../urls.txt"

test_passed=true

# Check that build exists
if [[ ! -f $load_balancer_executable ]]; then
    echo -e "${RED}Load balancer executable not found. Did you build it?${NC}"
    exit 1
fi

# Test simple load balancer with 3/3 healthy backend servers
# ------------------------------------------------------------------------------

# Arrange ----------------------------------------------------------------------
current_dir=$(pwd)
cd ../../../rust

echo -e "${GREEN}Starting backend servers...${NC}"
cargo run -p be -- -n "backend1" -p 8081 > /dev/null 2>&1 &
backend1_pid=$!
wait_for_server "backend1" 8081

cargo run -p be -- -n "backend2" -p 8082 > /dev/null 2>&1 &
backend2_pid=$!
wait_for_server "backend2" 8082

cargo run -p be -- -n "backend3" -p 8083 &> /dev/null 2>&1 &
backend3_pid=$!
wait_for_server "backend3" 8083

cd $current_dir

echo -e "${GREEN}Starting load balancer...${NC}"
$load_balancer_executable -c 10 -b "http://localhost:8081/", "http://localhost:8082/", "http://localhost:8083/" &> /dev/null 2>&1 &
lb_pid=$!
wait_for_server "load balancer" 8080

# Act --------------------------------------------------------------------------
echo -e "${GREEN}Running tests...${NC}"
result=$(curl --parallel --parallel-immediate --parallel-max 3 --config $urls_file --write-out "\n") 
count_backend1=$(echo $result | grep -o "backend1" | wc -l)
count_backend2=$(echo $result | grep -o "backend2" | wc -l)
count_backend3=$(echo $result | grep -o "backend3" | wc -l)

# Assert -----------------------------------------------------------------------
if [[ $count_backend1 -gt 0 && $count_backend1 -eq $count_backend2 && $count_backend2 -eq $count_backend3 ]]; then
    echo -e "${GREEN}All backend servers received the same number of requests.${NC}"
else
    echo -e "${RED}Did not receive the expected amount of answers
    backend1=${count_backend1}, backend2=${count_backend2},
    backend3=${count_backend3},.${NC}"
    test_passed=false
fi

echo -e "${YELLOW}Killing backend servers and load balancer...${NC}"
kill_pids $backend1_pid $backend2_pid $backend3_pid $lb_pid

# Test simple load balancer with 1/3 healthy backend servers
# ------------------------------------------------------------------------------

# Arrange ----------------------------------------------------------------------
current_dir=$(pwd)
cd ../../../rust

echo -e "${GREEN}Starting backend servers...${NC}"
cargo run -p be -- -n "backend1" -p 8081 > /dev/null 2>&1 &
backend1_pid=$!
wait_for_server "backend1" 8081

cd $current_dir

echo -e "${GREEN}Starting load balancer...${NC}"
$load_balancer_executable -c 10 -b "http://localhost:8081/", "http://localhost:8082/", "http://localhost:8083/" &> /dev/null 2>&1 &
lb_pid=$!
wait_for_server "load balancer" 8080

# Act --------------------------------------------------------------------------
echo -e "${GREEN}Running tests...${NC}"
result=$(curl --parallel --parallel-immediate --parallel-max 3 --config $urls_file --write-out "\n") 
count_backend1=$(echo $result | grep -o "backend1" | wc -l)
count_backend2=$(echo $result | grep -o "backend2" | wc -l)
count_backend3=$(echo $result | grep -o "backend3" | wc -l)

# Assert -----------------------------------------------------------------------
if [[ $count_backend1 -gt 0 && $count_backend2 -eq 0 && $count_backend3 -eq 0 ]]; then
    echo -e "${GREEN}Only received answers from backend 1.${NC}"
else
    echo -e "${RED}Did not receive the expected amount of answers
    backend1=${count_backend1}, backend2=${count_backend2},
    backend3=${count_backend3},.${NC}"
    test_passed=false
fi

echo -e "${YELLOW}Killing backend servers and load balancer...${NC}"
kill_pids $backend1_pid $lb_pid


# Test simple load balancer with 0/3 healthy backend servers
# ------------------------------------------------------------------------------

# Arrange ----------------------------------------------------------------------
echo -e "${GREEN}Starting load balancer only...${NC}"
$load_balancer_executable -c 10 -b "http://localhost:8081/", "http://localhost:8082/", "http://localhost:8083/" &> /dev/null 2>&1 &
lb_pid=$!
wait_for_server "load balancer" 8080

# Act --------------------------------------------------------------------------
echo -e "${GREEN}Running tests...${NC}"
result=$(curl --parallel --parallel-immediate --parallel-max 3 --config $urls_file --write-out "\n") 
count_backend1=$(echo $result | grep -o "backend1" | wc -l)
count_backend2=$(echo $result | grep -o "backend2" | wc -l)
count_backend3=$(echo $result | grep -o "backend3" | wc -l)

# Assert -----------------------------------------------------------------------
if [[ $count_backend1 -eq 0 && $count_backend2 -eq 0 && $count_backend3 -eq 0 ]]; then
    echo -e "${GREEN}Received no answers.${NC}"
else
    echo -e "${RED}Did not receive the expected amount of answers
    backend1=${count_backend1}, backend2=${count_backend2},
    backend3=${count_backend3},.${NC}"
    test_passed=false
fi

echo -e "${YELLOW}Killing backend servers and load balancer...${NC}"
kill_pids $backend1_pid $lb_pid

rm -rf ./uploads

echo "-------------------------------------------------------------------"
if [[ "$test_passed" == true ]]; then
    echo -e "${GREEN}Test passed.${NC}"
    exit 0
else
    echo -e "${RED}Test failed.${NC}"
    exit 1
fi

