#!/bin/bash

BIN_TARGET="./target/debug/bpxdbg"

for file in test/available/*; do
    if [[ -f $file ]]; then
        if [[ ${file} != *"."* ]]; then
            export ADDITIONAL=""
            export PRE_COMMAND=""
            export POST_COMMAND=""
            source $file
            echo "Running test: ${NAME} - ${DESC}"
            if [ "$PRE_COMMAND" ]; then
                echo "Running pre-command: ${PRE_COMMAND}"
                ${PRE_COMMAND}
            fi
            ${BIN_TARGET} ${COMMAND} > /tmp/mybin.stdout 2> /tmp/mybin.stderr
            if [[ $? != ${STATUS} ]]; then
                echo "Bad exit status: expected ${STATUS}, got $?"
                echo -e "\e[1;31m\t> Test Failure\e[0m"
                exit 1
            fi
            STDOUT_FILE="${file}.stdout"
            STDERR_FILE="${file}.stderr"
            if [[ -f ${STDOUT_FILE} ]]; then
                diff /tmp/mybin.stdout ${STDOUT_FILE}
                if [[ $? != 0 ]]; then
                    echo -e "\e[1;31m\t> Test Failure\e[0m"
                    exit 1
                fi
            fi
            if [[ -f ${STDERR_FILE} ]]; then
                diff /tmp/mybin.stderr ${STDERR_FILE}
                if [[ $? != 0 ]]; then
                    echo -e "\e[1;31m\t> Test Failure\e[0m"
                    exit 1
                fi
            fi
            if [ "$ADDITIONAL" ]; then
                ${ADDITIONAL}
                if [[ $? != 0 ]]; then
                    echo -e "\e[1;31m\t> Test Failure\e[0m"
                    exit 1
                fi
            fi
            if [ "$POST_COMMAND" ]; then
                echo "Running post-command: ${POST_COMMAND}"
                ${POST_COMMAND}
            fi
            echo -e "\e[1;32m\t> Test Passed\e[0m"
        fi
    fi
done
