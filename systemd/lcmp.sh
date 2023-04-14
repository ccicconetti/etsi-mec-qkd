#!/bin/bash

./lcmp \
	--address 0.0.0.0:8080 \
	--workers 5 \
	--app-list-type "static;file=application_list.json" \
	--app-context-type "single;10,URI"
