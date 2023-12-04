#!/usr/bin/env bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

dot $SCRIPT_DIR/nav_graph.dot -Tsvg -o $SCRIPT_DIR/nav_graph.svg
