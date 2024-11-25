set -ue
set -o pipefail
export BUILD_ARG_one="widget"
export BUILD_ARG_two="acme"
VALS=()
printenv | grep BUILD_ARG | while read line;do
  VALS+=($line)
done
echo "${VALS[*]}"
