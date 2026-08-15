#!/usr/bin/env bash

set -euo pipefail

packages=('git' 'gcc' 'tar' 'gzip' 'libreadline5' 'make' 'zlib1g' 'zlib1g-dev' 'flex' 'bison' 'perl' 'python3' 'tcl' 'gettext' 'odbc-postgresql' 'libreadline6-dev')
rfolder='/postgres'
dfolder='/postgres/data'
gitloc='git://git.postgresql.org/git/postgresql.git'
sysuser='postgres'
logfile='psqlinstall-log'

brew install postgresql

exit 0

